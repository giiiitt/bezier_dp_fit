use crate::fitting::{BezierFitter, FitError};
use crate::geometry::{Point2D, QuadraticBezier};
use rayon::prelude::*;
use std::collections::HashMap;

use super::config::FitConfig;

#[derive(Debug, Clone)]
pub struct FitResult {
    pub curves: Vec<QuadraticBezier>,
    pub total_error: f64,
    pub num_segments: usize,
    pub config: FitConfig,
}

impl FitResult {
    /// 杞崲涓?SVG 璺緞瀛楃涓?
    pub fn to_svg_path(&self) -> String {
        if self.curves.is_empty() {
            return String::new();
        }

        let mut path = format!("M {:.2} {:.2}", self.curves[0].p0.x, self.curves[0].p0.y);
        
        for curve in &self.curves {
            path.push(' ');
            path.push_str(&curve.to_svg_command());
        }

        path
    }

    /// 鑾峰彇鎵€鏈夋帶鍒剁偣
    pub fn control_points(&self) -> Vec<[(f64, f64); 3]> {
        self.curves.iter().map(|c| c.control_points()).collect()
    }

    /// 閲囨牱鎴愬瘑闆嗙偣闆?
    pub fn sample_points(&self, points_per_segment: usize) -> Vec<(f64, f64)> {
        self.curves
            .iter()
            .flat_map(|c| {
                c.sample(points_per_segment)
                    .into_iter()
                    .map(|p| (p.x, p.y))
            })
            .collect()
    }

    /// 杞崲涓?JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl serde::Serialize for FitResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("FitResult", 4)?;
        state.serialize_field("curves", &self.curves)?;
        state.serialize_field("total_error", &self.total_error)?;
        state.serialize_field("num_segments", &self.num_segments)?;
        state.serialize_field("config", &self.config)?;
        state.end()
    }
}

pub struct DPOptimizer;

impl DPOptimizer {
    /// 涓讳紭鍖栧嚱鏁?
    pub fn optimize(points: &[Point2D], config: &FitConfig) -> FitResult {
        let n = points.len();

        // 杈圭晫妫€鏌?
        if n == 0 {
            return FitResult {
                curves: vec![],
                total_error: 0.0,
                num_segments: 0,
                config: config.clone(),
            };
        }

        if n <= config.min_segment_len {
            // 鐐瑰お灏戞垨鍒氬ソ锛岀洿鎺ユ嫙鍚堜竴娈?
            let fit = BezierFitter::fit_segment(points);
            return FitResult {
                curves: vec![fit.bezier],
                total_error: fit.error,
                num_segments: 1,
                config: config.clone(),
            };
        }

        // 绗竴姝ワ細骞惰棰勮绠楁墍鏈夊彲鑳藉尯闂寸殑璇樊
        let error_cache = Self::compute_error_cache(points, config);

        // 绗簩姝ワ細DP
        let mut seg_dp = vec![usize::MAX; n];
        let mut err_dp = vec![f64::INFINITY; n];
        let mut parent = vec![0; n];
        seg_dp[0] = 0;
        err_dp[0] = 0.0;

        let max_len = config.max_segment_len.max(1);
        for i in config.min_segment_len..n {
            let start = i.saturating_sub(max_len - 1);
            let end = if config.min_segment_len > 0 {
                i.saturating_sub(config.min_segment_len - 1)
            } else {
                i  // 杈圭晫淇濇姢
            };

            for j in start..=end {
                if let Some(fit) = error_cache.get(&(j, i)) {
                    if fit.error > config.max_error {
                        continue; // 鍓灊
                    }

                    if seg_dp[j] == usize::MAX {
                        continue;
                    }
                    let cand_seg = seg_dp[j] + 1;
                    let cand_err = err_dp[j] + fit.error;
                    if cand_seg < seg_dp[i] || (cand_seg == seg_dp[i] && cand_err < err_dp[i]) {
                        seg_dp[i] = cand_seg;
                        err_dp[i] = cand_err;
                        parent[i] = j;
                    }
                }
            }
        }

        // 绗笁姝ワ細鍥炴函璺緞
        let total_error = err_dp[n - 1];
        
        // 妫€鏌ユ槸鍚︽壘鍒版湁鏁堣矾寰?
        if total_error.is_infinite() {
            // 娌℃湁鎵惧埌绗﹀悎璇樊瑕佹眰鐨勮矾寰勶紝浣跨敤瀹芥澗鐨勮宸噸璇?
            eprintln!("Warning: No valid path found with max_error={:.2}, using fallback", config.max_error);
            let fallback_config = FitConfig::new_clamped(
                config.min_segment_len,
                config.max_segment_len,
                f64::INFINITY  // 涓嶉檺鍒惰宸?
            );
            return Self::optimize(points, &fallback_config);
        }
        
        let curves = Self::reconstruct_curves(n - 1, &parent, &error_cache);
        let num_segments = curves.len();

        FitResult {
            curves,
            total_error,
            num_segments,
            config: config.clone(),
        }
    }

    /// 骞惰璁＄畻鎵€鏈夊尯闂寸殑璇樊
    fn compute_error_cache(
        points: &[Point2D],
        config: &FitConfig,
    ) -> HashMap<(usize, usize), FitError> {
        let n = points.len();
        let mut intervals = Vec::new();

        // 鐢熸垚鎵€鏈夐渶瑕佽绠楃殑鍖洪棿
        let max_len = config.max_segment_len.max(1);
        for i in config.min_segment_len..n {
            let start = i.saturating_sub(max_len - 1);
            let end = if config.min_segment_len > 0 {
                i.saturating_sub(config.min_segment_len - 1)
            } else {
                i
            };
            for j in start..=end {
                intervals.push((j, i));
            }
        }

        // 骞惰璁＄畻
        let results: Vec<_> = intervals
            .par_iter()
            .map(|&(start, end)| {
                let segment = &points[start..=end];
                let fit = BezierFitter::fit_segment_with_limit(segment, config.max_error);
                ((start, end), fit)
            })
            .collect();

        results.into_iter().collect()
    }

    /// 鍥炴函鏋勫缓鏇茬嚎搴忓垪
    fn reconstruct_curves(
        mut end: usize,
        parent: &[usize],
        cache: &HashMap<(usize, usize), FitError>,
    ) -> Vec<QuadraticBezier> {
        let mut segments = Vec::new();
        
        while end > 0 {
            let start = parent[end];
            if let Some(fit) = cache.get(&(start, end)) {
                segments.push(fit.bezier);
            } else {
                // 鐞嗚涓婁笉搴旇鍙戠敓锛屼絾涓轰簡鍋ュ．鎬?
                eprintln!("Warning: segment ({}, {}) not found in cache", start, end);
            }
            end = start;
        }

        segments.reverse();
        segments
    }
}

/// 渚挎嵎鍑芥暟
pub fn fit_curve(points: &[Point2D], config: &FitConfig) -> FitResult {
    DPOptimizer::optimize(points, config)
}

