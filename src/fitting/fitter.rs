use crate::geometry::{Point2D, QuadraticBezier};

#[derive(Debug, Clone)]
pub struct FitError {
    pub bezier: QuadraticBezier,
    pub error: f64,
}

pub struct BezierFitter;

impl BezierFitter {
    pub(crate) fn compute_bezier(points: &[Point2D]) -> QuadraticBezier {
        let n = points.len();

        if n == 0 {
            let p = Point2D::new(0.0, 0.0);
            return QuadraticBezier::new(p, p, p);
        }

        if n == 1 {
            let p = points[0];
            return QuadraticBezier::new(p, p, p);
        }

        if n == 2 {
            let p0 = points[0];
            let p2 = points[1];
            let p1 = p0.lerp(&p2, 0.5);
            return QuadraticBezier::new(p0, p1, p2);
        }

        let p0 = points[0];
        let p2 = points[n - 1];
        let t_values = Self::compute_t_values(points);

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_weight = 0.0;

        for i in 0..n {
            let t = t_values[i];
            let mt = 1.0 - t;
            let weight = 2.0 * mt * t;

            if weight.abs() < 1e-10 {
                continue;
            }

            let target_x = points[i].x - mt * mt * p0.x - t * t * p2.x;
            let target_y = points[i].y - mt * mt * p0.y - t * t * p2.y;

            sum_x += weight * target_x;
            sum_y += weight * target_y;
            sum_weight += weight * weight;
        }

        let p1 = if sum_weight > 1e-10 {
            Point2D::new(sum_x / sum_weight, sum_y / sum_weight)
        } else {
            p0.lerp(&p2, 0.5)
        };

        QuadraticBezier::new(p0, p1, p2)
    }

    pub fn fit_segment(points: &[Point2D]) -> FitError {
        let bezier = Self::compute_bezier(points);
        let error = Self::compute_error(&bezier, points);
        FitError { bezier, error }
    }

    pub fn fit_segment_with_limit(points: &[Point2D], max_error: f64) -> FitError {
        let bezier = Self::compute_bezier(points);
        let error = Self::compute_error_with_limit(&bezier, points, max_error);
        FitError { bezier, error }
    }

    fn compute_t_values(points: &[Point2D]) -> Vec<f64> {
        let n = points.len();
        let mut t_values = vec![0.0; n];
        let mut distances = vec![0.0; n];

        for i in 1..n {
            distances[i] = distances[i - 1] + points[i].distance_to(&points[i - 1]);
        }

        let total_length = distances[n - 1];
        if total_length < 1e-10 {
            return (0..n)
                .map(|i| i as f64 / (n - 1).max(1) as f64)
                .collect();
        }

        for i in 0..n {
            t_values[i] = distances[i] / total_length;
        }

        t_values
    }

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

    pub fn compute_error_with_limit(
        bezier: &QuadraticBezier,
        points: &[Point2D],
        max_error: f64,
    ) -> f64 {
        if points.is_empty() {
            return 0.0;
        }

        let n = points.len() as f64;
        let max_sum = max_error * n;
        let mut sum = 0.0;

        for p in points {
            sum += bezier.distance_to_point(p).powi(2);
            if sum > max_sum {
                return sum / n;
            }
        }

        sum / n
    }
}
