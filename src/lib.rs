pub mod geometry;
pub mod fitting;
pub mod optimizer;
mod python;

// 导出主要类型
pub use geometry::{Point2D, QuadraticBezier};
pub use fitting::{BezierFitter, FitError};
pub use optimizer::{FitConfig, FitResult, DPOptimizer, fit_curve};

// Python模块入口
use pyo3::prelude::*;

#[pymodule]
fn bezier_dp_fit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(python::bindings::fit_curve_py, m)?)?;
    m.add_class::<python::bindings::PyFitResult>()?;
    Ok(())
}