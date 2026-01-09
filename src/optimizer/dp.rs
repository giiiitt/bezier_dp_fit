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
    /// 转换为 SVG 路径字符串
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

    /// 获取所有控制点
    pub fn control_points(&self) -> Vec<[(f64, f64); 3]> {
        self.curves.iter().map(|c| c.control_points()).collect()
    }

    /// 采样成密集点集
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

    /// 转换为 JSON
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
    /// 主优化函数
    pub fn optimize(points: &[Point2D], config: &FitConfig) -> FitResult {
        let n = points.len();

        // 边界检查
        if n == 0 {
            return FitResult {
                curves: vec![],
                total_error: 0.0,
                num_segments: 0,
                config: config.clone(),
            };
        }

        if n <= config.min_segment_len {
            // 点太少或刚好，直接拟合一段
            let fit = BezierFitter::fit_segment(points);
            return FitResult {
                curves: vec![fit.bezier],
                total_error: fit.error,
                num_segments: 1,
                config: config.clone(),
            };
        }

        // 第一步：并行预计算所有可能区间的误差
        let error_cache = Self::compute_error_cache(points, config);

        // 第二步：DP
        let mut dp = vec![f64::INFINITY; n];
        let mut parent = vec![0; n];
        dp[0] = 0.0;

        for i in config.min_segment_len..n {
            let start = i.saturating_sub(config.max_segment_len);
            let end = if config.min_segment_len > 0 {
                i.saturating_sub(config.min_segment_len - 1)
            } else {
                i  // 边界保护
            };

            for j in start..=end {
                if let Some(fit) = error_cache.get(&(j, i)) {
                    if fit.error > config.max_error {
                        continue; // 剪枝
                    }

                    let cost = dp[j] + fit.error;
                    if cost < dp[i] {
                        dp[i] = cost;
                        parent[i] = j;
                    }
                }
            }
        }

        // 第三步：回溯路径
        let total_error = dp[n - 1];
        
        // 检查是否找到有效路径
        if total_error.is_infinite() {
            // 没有找到符合误差要求的路径，使用宽松的误差重试
            eprintln!("Warning: No valid path found with max_error={:.2}, using fallback", config.max_error);
            let fallback_config = FitConfig::new_clamped(
                config.min_segment_len,
                config.max_segment_len,
                f64::INFINITY  // 不限制误差
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

    /// 并行计算所有区间的误差
    fn compute_error_cache(
        points: &[Point2D],
        config: &FitConfig,
    ) -> HashMap<(usize, usize), FitError> {
        let n = points.len();
        let mut intervals = Vec::new();

        // 生成所有需要计算的区间
        for i in config.min_segment_len..n {
            let start = i.saturating_sub(config.max_segment_len);
            let end = if config.min_segment_len > 0 {
                i.saturating_sub(config.min_segment_len - 1)
            } else {
                i
            };
            for j in start..=end {
                intervals.push((j, i));
            }
        }

        // 并行计算
        let results: Vec<_> = intervals
            .par_iter()
            .map(|&(start, end)| {
                let segment = &points[start..=end];
                let fit = BezierFitter::fit_segment(segment);
                ((start, end), fit)
            })
            .collect();

        results.into_iter().collect()
    }

    /// 回溯构建曲线序列
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
                // 理论上不应该发生，但为了健壮性
                eprintln!("Warning: segment ({}, {}) not found in cache", start, end);
            }
            end = start;
        }

        segments.reverse();
        segments
    }
}

/// 便捷函数
pub fn fit_curve(points: &[Point2D], config: &FitConfig) -> FitResult {
    DPOptimizer::optimize(points, config)
}