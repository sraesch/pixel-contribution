use arrayvec::ArrayVec;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use nalgebra_glm::{zero, Vec2};
use pixel_contrib::polygon_2d::Polygon2D;

#[derive(Debug, Clone)]
struct BenchmarkInput<const N: usize> {
    pub polygon: Polygon2D<N>,
    pub width: f32,
    pub height: f32,
}

impl<const N: usize> BenchmarkInput<N> {
    #[inline]
    pub fn go(&self) {
        self.polygon
            .compute_area_with_overlapping_rectangle(self.width, self.height);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let input1 = BenchmarkInput {
        polygon: Polygon2D::new([
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ]),
        width: 1.0,
        height: 1.0,
    };

    let input2 = BenchmarkInput {
        polygon: Polygon2D::new([
            Vec2::new(-1.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-2.0, 1.0),
        ]),
        width: 1.0,
        height: 1.0,
    };

    let input3 = BenchmarkInput {
        polygon: {
            const N: usize = 4;
            let mut vertices = [zero(); N];

            vertices.iter_mut().enumerate().for_each(|(i, v)| {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / N as f32;
                *v = Vec2::new(angle.cos(), angle.sin() + 1f32);
            });

            Polygon2D::new(vertices)
        },
        width: 1f32,
        height: 2f32,
    };

    let input4 = BenchmarkInput {
        polygon: {
            const N: usize = 8;
            let mut vertices = [zero(); N];

            vertices.iter_mut().enumerate().for_each(|(i, v)| {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / N as f32;
                *v = Vec2::new(angle.cos(), angle.sin() + 1f32);
            });

            Polygon2D::new(vertices)
        },
        width: 1f32,
        height: 2f32,
    };

    const N: usize = 8;
    let input5: [BenchmarkInput<N>; 4] = {
        const WIDTH: f32 = 2f32;
        const HEIGHT: f32 = 2f32;

        let corners = [
            Vec2::new(0f32, 0f32),
            Vec2::new(WIDTH, 0f32),
            Vec2::new(WIDTH, HEIGHT),
            Vec2::new(0f32, HEIGHT),
        ];

        let circles: ArrayVec<BenchmarkInput<N>, 4> =
            ArrayVec::from_iter(corners.iter().map(|corner| {
                let mut vertices = [zero(); N];

                vertices.iter_mut().enumerate().for_each(|(i, v)| {
                    let angle = 2.0 * std::f32::consts::PI * i as f32 / N as f32;
                    *v = Vec2::new(corner.x + angle.cos(), corner.y + angle.sin());
                });

                BenchmarkInput {
                    polygon: Polygon2D::new(vertices),
                    width: WIDTH,
                    height: HEIGHT,
                }
            }));

        circles.into_inner().unwrap()
    };

    let mut group = c.benchmark_group("compute_area_with_overlapping_rectangle");

    group.bench_with_input(
        BenchmarkId::new("compute_area_with_overlapping_rectangle", "input1"),
        &input1,
        |b, input| {
            b.iter(|| {
                input.go();
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("compute_area_with_overlapping_rectangle", "input2"),
        &input2,
        |b, input| {
            b.iter(|| {
                input.go();
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("compute_area_with_overlapping_rectangle", "input3"),
        &input3,
        |b, input| {
            b.iter(|| {
                input.go();
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("compute_area_with_overlapping_rectangle", "input4"),
        &input4,
        |b, input| {
            b.iter(|| {
                input.go();
            });
        },
    );

    for (i, input) in input5.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new(
                "compute_area_with_overlapping_rectangle",
                format!("input5_{}", i),
            ),
            input,
            |b, input| {
                b.iter(|| {
                    input.go();
                });
            },
        );
    }

    group.finish();

    // c.bench_function("overlapping rectangle", |b| {
    //     b.iter(|| {
    //         black_box(&rectangle)
    //             .compute_area_with_overlapping_rectangle(black_box(1.0), black_box(1.0))
    //     });
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
