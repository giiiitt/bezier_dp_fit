use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bezier_dp_fit::{Point2D, FitConfig, fit_curve};

fn generate_points(n: usize) -> Vec<Point2D> {
    (0..n)
        .map(|i| {
            let x = i as f64;
            let y = (x * 0.1).sin() * 50.0 + x * 0.5;
            Point2D::new(x, y)
        })
        .collect()
}

fn benchmark_fitting(c: &mut Criterion) {
    let mut group = c.benchmark_group("bezier_fitting");
    
    for size in [100, 500, 1000, 2000].iter() {
        let points = generate_points(*size);
        let config = FitConfig::default();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    fit_curve(black_box(&points), black_box(&config))
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_segment_lengths(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_lengths");
    let points = generate_points(1000);
    
    for min_len in [10, 30, 50].iter() {
        let config = FitConfig::new(*min_len, min_len * 6, 2.0);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(min_len),
            min_len,
            |b, _| {
                b.iter(|| {
                    fit_curve(black_box(&points), black_box(&config))
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_fitting, benchmark_segment_lengths);
criterion_main!(benches);