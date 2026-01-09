use pyo3::prelude::*;
use pyo3::types::PyList;
use numpy::{PyArray2, PyArrayMethods, PyUntypedArrayMethods};

use crate::geometry::Point2D;
use crate::optimizer::{FitConfig, fit_curve};

#[pyclass]
#[derive(Clone)]
pub struct PyFitResult {
    #[pyo3(get)]
    pub total_error: f64,
    #[pyo3(get)]
    pub num_segments: usize,
    inner: crate::optimizer::FitResult,
}

#[pymethods]
impl PyFitResult {
    /// 获取控制点列表
    fn control_points(&self) -> Vec<Vec<(f64, f64)>> {
        self.inner
            .control_points()
            .into_iter()
            .map(|points| points.to_vec())
            .collect()
    }

    /// 转换为SVG路径
    fn to_svg(&self) -> String {
        self.inner.to_svg_path()
    }

    /// 采样点
    fn sample_points(&self, points_per_segment: usize) -> Vec<(f64, f64)> {
        self.inner.sample_points(points_per_segment)
    }

    /// 转JSON
    fn to_json(&self) -> PyResult<String> {
        self.inner
            .to_json()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "FitResult(segments={}, error={:.2})",
            self.num_segments, self.total_error
        )
    }
}

/// Python接口：拟合曲线
#[pyfunction]
#[pyo3(signature = (points, min_segment_len=30, max_segment_len=200, max_error=2.0))]
pub fn fit_curve_py(
    points: &Bound<'_, PyAny>,
    min_segment_len: usize,
    max_segment_len: usize,
    max_error: f64,
) -> PyResult<PyFitResult> {
    // 解析输入点
    let pts = parse_points(points)?;

    // 配置（自动修正无效参数）
    let config = FitConfig::new_clamped(min_segment_len, max_segment_len, max_error);

    // 拟合
    let result = fit_curve(&pts, &config);

    Ok(PyFitResult {
        total_error: result.total_error,
        num_segments: result.num_segments,
        inner: result,
    })
}

/// 解析Python输入的点（支持列表和numpy数组）
fn parse_points(obj: &Bound<'_, PyAny>) -> PyResult<Vec<Point2D>> {
    // 尝试作为numpy数组
    if let Ok(arr) = obj.cast::<PyArray2<f64>>() {
        let readonly = arr.readonly();
        let shape = readonly.shape();
        let (rows, cols) = match shape {
            [rows, cols] => (*rows, *cols),
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Numpy array must have shape (N, 2)",
                ));
            }
        };

        if cols != 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Numpy array must have shape (N, 2)",
            ));
        }

        let mut points = Vec::with_capacity(rows);
        for i in 0..rows {
            points.push(Point2D::new(
                *readonly.get([i, 0]).unwrap(),
                *readonly.get([i, 1]).unwrap(),
            ));
        }
        return Ok(points);
    }

    // 尝试作为列表
    if let Ok(list) = obj.cast::<PyList>() {
        let mut points = Vec::with_capacity(list.len());
        for item in list.iter() {
            if let Ok(tuple) = item.extract::<(f64, f64)>() {
                points.push(Point2D::new(tuple.0, tuple.1));
            } else if let Ok(list) = item.extract::<Vec<f64>>() {
                if list.len() == 2 {
                    points.push(Point2D::new(list[0], list[1]));
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Each point must have 2 coordinates",
                    ));
                }
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Invalid point format",
                ));
            }
        }
        return Ok(points);
    }

    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
        "Points must be a list of tuples or numpy array",
    ))
}

// Python模块定义在 lib.rs 中，这里不需要重复定义
