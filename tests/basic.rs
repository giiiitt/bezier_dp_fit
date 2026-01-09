use bezier_dp_fit::{Point2D, FitConfig, fit_curve};

#[test]
fn test_simple_line() {
    // 测试直线拟合
    let points: Vec<Point2D> = (0..50)
        .map(|i| Point2D::new(i as f64, i as f64))
        .collect();

    let config = FitConfig::new(10, 50, 2.0);
    let result = fit_curve(&points, &config);

    assert!(result.num_segments >= 1);
    assert!(result.total_error < 10.0);
    println!("直线测试: {} 段, 误差 {:.2}", result.num_segments, result.total_error);
}

#[test]
fn test_parabola() {
    // 测试抛物线拟合
    let points: Vec<Point2D> = (0..100)
        .map(|i| {
            let x = i as f64;
            let y = 0.01 * x * x;
            Point2D::new(x, y)
        })
        .collect();

    let config = FitConfig::new(15, 80, 5.0);
    let result = fit_curve(&points, &config);

    assert!(result.num_segments >= 1);
    println!("抛物线测试: {} 段, 误差 {:.2}", result.num_segments, result.total_error);
}

#[test]
fn test_svg_output() {
    let points: Vec<Point2D> = vec![
        Point2D::new(0.0, 0.0),
        Point2D::new(10.0, 10.0),
        Point2D::new(20.0, 15.0),
        Point2D::new(30.0, 10.0),
        Point2D::new(40.0, 0.0),
    ];

    let config = FitConfig::new(2, 10, 5.0);
    let result = fit_curve(&points, &config);

    let svg = result.to_svg_path();
    assert!(svg.starts_with("M"));
    assert!(svg.contains("Q"));
    println!("SVG: {}", svg);
}

#[test]
fn test_control_points() {
    let points: Vec<Point2D> = (0..30)
        .map(|i| Point2D::new(i as f64, (i as f64).sin() * 10.0))
        .collect();

    let config = FitConfig::new(5, 20, 2.0);
    let result = fit_curve(&points, &config);

    let cp = result.control_points();
    assert_eq!(cp.len(), result.num_segments);
    
    for (i, points) in cp.iter().enumerate() {
        println!("段{}: {:?}", i, points);
    }
}