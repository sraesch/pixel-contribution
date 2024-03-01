use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nalgebra_glm::Vec2;
use pixel_contrib::polygon_2d::Polygon2D;

fn criterion_benchmark(c: &mut Criterion) {
    let rectangle = Polygon2D::new([
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
    ]);

    c.bench_function("overlapping rectangle", |b| {
        b.iter(|| {
            black_box(&rectangle)
                .compute_area_with_overlapping_rectangle(black_box(1.0), black_box(1.0))
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
