#![cfg(feature = "cuda")]

use std::collections::HashMap;

use cudarc::driver::{CudaDevice, DeviceRepr, LaunchAsync, LaunchConfig};
use cudarc::nvrtc::compile_ptx;

use crate::fitting::{BezierFitter, FitError};
use crate::geometry::Point2D;
use crate::optimizer::config::FitConfig;

const CUDA_SRC: &str = r#"
extern "C" __global__ void compute_errors(
    const double* pts_x,
    const double* pts_y,
    int n_points,
    const double* p0x,
    const double* p0y,
    const double* p1x,
    const double* p1y,
    const double* p2x,
    const double* p2y,
    const int* start_idx,
    const int* end_idx,
    double max_error,
    double* out_err,
    int n_segments
) {
    int idx = (int)(blockIdx.x * blockDim.x + threadIdx.x);
    if (idx >= n_segments) {
        return;
    }

    int start = start_idx[idx];
    int end = end_idx[idx];
    int len = end - start + 1;
    if (len <= 0) {
        out_err[idx] = 0.0;
        return;
    }

    double p0xv = p0x[idx];
    double p0yv = p0y[idx];
    double p1xv = p1x[idx];
    double p1yv = p1y[idx];
    double p2xv = p2x[idx];
    double p2yv = p2y[idx];

    double dx01 = p0xv - p1xv;
    double dy01 = p0yv - p1yv;
    double dx12 = p1xv - p2xv;
    double dy12 = p1yv - p2yv;
    double curve_len = sqrt(dx01 * dx01 + dy01 * dy01)
                     + sqrt(dx12 * dx12 + dy12 * dy12);

    int samples = (int)(curve_len / 2.0);
    if (samples < 50) samples = 50;
    if (samples > 200) samples = 200;
    double denom = (samples > 1) ? (double)(samples - 1) : 1.0;

    double max_sum = max_error * (double)len;
    double sum = 0.0;

    for (int i = start; i <= end; ++i) {
        double px = pts_x[i];
        double py = pts_y[i];
        double min_d2 = 1.0e300;

        for (int s = 0; s < samples; ++s) {
            double t = (double)s / denom;
            double mt = 1.0 - t;
            double mt2 = mt * mt;
            double t2 = t * t;

            double bx = mt2 * p0xv + 2.0 * mt * t * p1xv + t2 * p2xv;
            double by = mt2 * p0yv + 2.0 * mt * t * p1yv + t2 * p2yv;

            double dx = bx - px;
            double dy = by - py;
            double d2 = dx * dx + dy * dy;
            if (d2 < min_d2) {
                min_d2 = d2;
            }
        }

        sum += min_d2;
        if (sum > max_sum) {
            break;
        }
    }

    out_err[idx] = sum / (double)len;
}
"#;

pub fn compute_error_cache_cuda(
    points: &[Point2D],
    config: &FitConfig,
) -> Result<HashMap<(usize, usize), FitError>, String> {
    let n = points.len();
    if n == 0 {
        return Ok(HashMap::new());
    }

    let mut starts: Vec<i32> = Vec::new();
    let mut ends: Vec<i32> = Vec::new();
    let mut p0x: Vec<f64> = Vec::new();
    let mut p0y: Vec<f64> = Vec::new();
    let mut p1x: Vec<f64> = Vec::new();
    let mut p1y: Vec<f64> = Vec::new();
    let mut p2x: Vec<f64> = Vec::new();
    let mut p2y: Vec<f64> = Vec::new();
    let mut beziers = Vec::new();

    let max_len = config.max_segment_len.max(1);
    for i in config.min_segment_len..n {
        let start = i.saturating_sub(max_len - 1);
        let end = if config.min_segment_len > 0 {
            i.saturating_sub(config.min_segment_len - 1)
        } else {
            i
        };
        for j in start..=end {
            let segment = &points[j..=i];
            let bezier = BezierFitter::compute_bezier(segment);
            starts.push(j as i32);
            ends.push(i as i32);
            p0x.push(bezier.p0.x);
            p0y.push(bezier.p0.y);
            p1x.push(bezier.p1.x);
            p1y.push(bezier.p1.y);
            p2x.push(bezier.p2.x);
            p2y.push(bezier.p2.y);
            beziers.push(bezier);
        }
    }

    let segment_count = starts.len();
    if segment_count == 0 {
        return Ok(HashMap::new());
    }

    let points_x: Vec<f64> = points.iter().map(|p| p.x).collect();
    let points_y: Vec<f64> = points.iter().map(|p| p.y).collect();

    let dev = CudaDevice::new(0).map_err(|e| format!("cuda init: {e}"))?;
    let ptx = compile_ptx(CUDA_SRC).map_err(|e| format!("nvrtc: {e}"))?;
    dev.load_ptx(ptx, "bezier", &["compute_errors"])
        .map_err(|e| format!("load ptx: {e}"))?;
    let func = dev
        .get_func("bezier", "compute_errors")
        .ok_or_else(|| "get func: compute_errors not found".to_string())?;

    let d_points_x = dev
        .htod_copy(points_x)
        .map_err(|e| format!("copy points x: {e}"))?;
    let d_points_y = dev
        .htod_copy(points_y)
        .map_err(|e| format!("copy points y: {e}"))?;
    let d_p0x = dev.htod_copy(p0x).map_err(|e| format!("copy p0x: {e}"))?;
    let d_p0y = dev.htod_copy(p0y).map_err(|e| format!("copy p0y: {e}"))?;
    let d_p1x = dev.htod_copy(p1x).map_err(|e| format!("copy p1x: {e}"))?;
    let d_p1y = dev.htod_copy(p1y).map_err(|e| format!("copy p1y: {e}"))?;
    let d_p2x = dev.htod_copy(p2x).map_err(|e| format!("copy p2x: {e}"))?;
    let d_p2y = dev.htod_copy(p2y).map_err(|e| format!("copy p2y: {e}"))?;
    let d_starts = dev
        .htod_copy(starts.clone())
        .map_err(|e| format!("copy starts: {e}"))?;
    let d_ends = dev
        .htod_copy(ends.clone())
        .map_err(|e| format!("copy ends: {e}"))?;
    let mut d_out = dev
        .alloc_zeros::<f64>(segment_count)
        .map_err(|e| format!("alloc output: {e}"))?;

    let block_dim = 128u32;
    let grid_dim = ((segment_count as u32) + block_dim - 1) / block_dim;
    let cfg = LaunchConfig {
        block_dim: (block_dim, 1, 1),
        grid_dim: (grid_dim, 1, 1),
        shared_mem_bytes: 0,
    };

    let n_points = points.len() as i32;
    let n_segments = segment_count as i32;
    let max_error = config.max_error;
    let mut args: Vec<*mut std::ffi::c_void> = vec![
        (&d_points_x).as_kernel_param(),
        (&d_points_y).as_kernel_param(),
        (&n_points).as_kernel_param(),
        (&d_p0x).as_kernel_param(),
        (&d_p0y).as_kernel_param(),
        (&d_p1x).as_kernel_param(),
        (&d_p1y).as_kernel_param(),
        (&d_p2x).as_kernel_param(),
        (&d_p2y).as_kernel_param(),
        (&d_starts).as_kernel_param(),
        (&d_ends).as_kernel_param(),
        (&max_error).as_kernel_param(),
        (&mut d_out).as_kernel_param(),
        (&n_segments).as_kernel_param(),
    ];

    unsafe {
        func.launch(cfg, &mut args)
            .map_err(|e| format!("launch: {e}"))?;
    }

    let errors = dev
        .dtoh_sync_copy(&d_out)
        .map_err(|e| format!("copy back: {e}"))?;

    let mut cache = HashMap::with_capacity(segment_count);
    for idx in 0..segment_count {
        let start = starts[idx] as usize;
        let end = ends[idx] as usize;
        cache.insert(
            (start, end),
            FitError {
                bezier: beziers[idx],
                error: errors[idx],
            },
        );
    }

    Ok(cache)
}
