use super::point::Point2D;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuadraticBezier {
    pub p0: Point2D,  // 起点
    pub p1: Point2D,  // 控制点
    pub p2: Point2D,  // 终点
}

impl QuadraticBezier {
    pub fn new(p0: Point2D, p1: Point2D, p2: Point2D) -> Self {
        Self { p0, p1, p2 }
    }

    /// 计算贝塞尔曲线上参数为 t 的点 (t ∈ [0, 1])
    pub fn evaluate(&self, t: f64) -> Point2D {
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        Point2D {
            x: mt2 * self.p0.x + 2.0 * mt * t * self.p1.x + t2 * self.p2.x,
            y: mt2 * self.p0.y + 2.0 * mt * t * self.p1.y + t2 * self.p2.y,
        }
    }

    /// 采样曲线上的点
    pub fn sample(&self, num_points: usize) -> Vec<Point2D> {
        (0..num_points)
            .map(|i| {
                let t = i as f64 / (num_points - 1).max(1) as f64;
                self.evaluate(t)
            })
            .collect()
    }

    /// 转换为 SVG 路径的 Q 指令
    pub fn to_svg_command(&self) -> String {
        format!(
            "Q {:.2} {:.2}, {:.2} {:.2}",
            self.p1.x, self.p1.y, self.p2.x, self.p2.y
        )
    }

    /// 获取控制点数组
    pub fn control_points(&self) -> [(f64, f64); 3] {
        [self.p0.into(), self.p1.into(), self.p2.into()]
    }

    /// 计算点到曲线的最近距离（近似）
    pub fn distance_to_point(&self, point: &Point2D) -> f64 {
        // 根据曲线长度自适应采样
        let curve_length = self.p0.distance_to(&self.p1) + self.p1.distance_to(&self.p2);
        let samples = (curve_length / 2.0).max(50.0).min(200.0) as usize;
        
        (0..samples)
            .map(|i| {
                let t = i as f64 / (samples - 1) as f64;
                let p = self.evaluate(t);
                p.distance_to(point)
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(f64::INFINITY)
    }
}