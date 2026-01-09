use crate::geometry::{Point2D, QuadraticBezier};

#[derive(Debug, Clone)]
pub struct FitError {
    pub bezier: QuadraticBezier,
    pub error: f64,
}

pub struct BezierFitter;

impl BezierFitter {
    /// 用最小二乘法拟合一段点
    pub fn fit_segment(points: &[Point2D]) -> FitError {
        let n = points.len();
        
        // 边界情况处理
        if n == 0 {
            // 空点集，返回零长度曲线
            let p = Point2D::new(0.0, 0.0);
            return FitError {
                bezier: QuadraticBezier::new(p, p, p),
                error: 0.0,
            };
        }

        if n == 1 {
            // 单点，返回退化曲线
            let p = points[0];
            return FitError {
                bezier: QuadraticBezier::new(p, p, p),
                error: 0.0,
            };
        }

        if n == 2 {
            // 两点，控制点在中点
            let p0 = points[0];
            let p2 = points[1];
            let p1 = p0.lerp(&p2, 0.5);
            return FitError {
                bezier: QuadraticBezier::new(p0, p1, p2),
                error: 0.0,
            };
        }
        
        if n < 3 {
            // 点太少，直接连线
            let p0 = points[0];
            let p2 = points[n - 1];
            let p1 = p0.lerp(&p2, 0.5);
            let bezier = QuadraticBezier::new(p0, p1, p2);
            return FitError {
                error: Self::compute_error(&bezier, points),
                bezier,
            };
        }

        // 固定起点和终点
        let p0 = points[0];
        let p2 = points[n - 1];

        // 为每个数据点分配参数 t (弦长参数化)
        let t_values = Self::compute_t_values(points);

        // 构建最小二乘方程: A * p1 = b
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_weight = 0.0;

        for i in 0..n {
            let t = t_values[i];
            let mt = 1.0 - t;
            let weight = 2.0 * mt * t; // B1(t) = 2(1-t)t

            if weight.abs() < 1e-10 {
                continue;
            }

            // 从观察值中减去已知的起点和终点贡献
            let target_x = points[i].x - mt * mt * p0.x - t * t * p2.x;
            let target_y = points[i].y - mt * mt * p0.y - t * t * p2.y;

            sum_x += weight * target_x;
            sum_y += weight * target_y;
            sum_weight += weight * weight;
        }

        // 求解控制点 p1
        let p1 = if sum_weight > 1e-10 {
            Point2D::new(sum_x / sum_weight, sum_y / sum_weight)
        } else {
            // 退化情况，使用中点
            p0.lerp(&p2, 0.5)
        };

        let bezier = QuadraticBezier::new(p0, p1, p2);
        let error = Self::compute_error(&bezier, points);

        FitError { bezier, error }
    }

    /// 计算参数化值 (弦长参数化)
    fn compute_t_values(points: &[Point2D]) -> Vec<f64> {
        let n = points.len();
        let mut t_values = vec![0.0; n];
        let mut distances = vec![0.0; n];

        // 计算累积弦长
        for i in 1..n {
            distances[i] = distances[i - 1] + points[i].distance_to(&points[i - 1]);
        }

        let total_length = distances[n - 1];
        if total_length < 1e-10 {
            // 所有点重合
            return (0..n).map(|i| i as f64 / (n - 1).max(1) as f64).collect();
        }

        // 归一化到 [0, 1]
        for i in 0..n {
            t_values[i] = distances[i] / total_length;
        }

        t_values
    }

    /// 计算拟合误差（点到曲线距离的平方和）
    pub fn compute_error(bezier: &QuadraticBezier, points: &[Point2D]) -> f64 {
        if points.is_empty() {
            return 0.0;
        }
        
        points
            .iter()
            .map(|p| bezier.distance_to_point(p).powi(2))
            .sum::<f64>()
            / points.len() as f64
    }
}