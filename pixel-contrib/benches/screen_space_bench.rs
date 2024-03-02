use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use math::BoundingSphere;
use nalgebra_glm::Mat4;
use pixel_contrib::screen_space::ScreenSpaceEstimator;

struct BenchmarkContextInput {
    pub estimator: ScreenSpaceEstimator,
}

impl BenchmarkContextInput {
    pub fn new(p: Mat4, m: Mat4, height: f32) -> Self {
        let mut estimator = ScreenSpaceEstimator::new();

        estimator.update_camera(m, p, height);

        Self { estimator }
    }

    #[inline]
    pub fn go(&self, sphere: BoundingSphere) {
        self.estimator
            .estimate_screen_space_for_bounding_sphere(sphere);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let height = 600f32;
    let sphere = BoundingSphere {
        center: nalgebra_glm::vec3(0f32, 0f32, 0f32),
        radius: std::f32::consts::SQRT_2,
    };

    let model_view1 = Mat4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -2.1213202, 1.0,
    )
    .transpose();
    let projection1 = Mat4::new(
        0.97583085, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.3999999, -1.0, 0.0, 0.0,
        -1.697056, 0.0,
    )
    .transpose();

    let model_view2 = Mat4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -3.6970365, 0.17255715,
        -3.4511428, 1.0,
    )
    .transpose();
    let projection2 = Mat4::new(
        0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -2.152261, -1.0, 0.0, 0.0, -6.4209323,
        0.0,
    )
    .transpose();

    let model_view3 = Mat4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.7484075, 1.0,
    )
    .transpose();
    let projection3 = Mat4::new(
        0.75,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.1890486,
        -1.0,
        0.0,
        0.0,
        -0.73156685,
        0.0,
    )
    .transpose();

    let model_view4 = Mat4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.8732692, -1.3417664,
        -2.2300825, 1.0,
    )
    .transpose();
    let projection4 = Mat4::new(
        0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.4615251, -1.0, 0.0, 0.0, -2.008282,
        0.0,
    )
    .transpose();

    let model_view5 = Mat4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.2909417, 1.0,
    )
    .transpose();
    let projection5 = Mat4::new(
        0.75,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.0000019,
        -1.0,
        0.0,
        0.0,
        -6.824531e-6,
        0.0,
    )
    .transpose();

    let model_view6 = Mat4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -10.286586, 1.3572578,
        -4.2860775, 1.0,
    )
    .transpose();
    let projection6 = Mat4::new(
        0.75, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -2.6245716, -1.0, 0.0, 0.0, -10.409276,
        0.0,
    )
    .transpose();

    let input1 = BenchmarkContextInput::new(projection1, model_view1, height);
    let input2 = BenchmarkContextInput::new(projection2, model_view2, height);
    let input3 = BenchmarkContextInput::new(projection3, model_view3, height);
    let input4 = BenchmarkContextInput::new(projection4, model_view4, height);
    let input5 = BenchmarkContextInput::new(projection5, model_view5, height);
    let input6 = BenchmarkContextInput::new(projection6, model_view6, height);

    let mut group = c.benchmark_group("estimate_screen_space_for_bounding_sphere");

    group.bench_with_input(
        BenchmarkId::new("estimate_screen_space_for_bounding_sphere", "input1"),
        &input1,
        |b, input| {
            b.iter(|| {
                input.go(black_box(sphere.clone()));
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("estimate_screen_space_for_bounding_sphere", "input2"),
        &input2,
        |b, input| {
            b.iter(|| {
                input.go(black_box(sphere.clone()));
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("estimate_screen_space_for_bounding_sphere", "input3"),
        &input3,
        |b, input| {
            b.iter(|| {
                input.go(black_box(sphere.clone()));
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("estimate_screen_space_for_bounding_sphere", "input4"),
        &input4,
        |b, input| {
            b.iter(|| {
                input.go(black_box(sphere.clone()));
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("estimate_screen_space_for_bounding_sphere", "input5"),
        &input5,
        |b, input| {
            b.iter(|| {
                input.go(black_box(sphere.clone()));
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("estimate_screen_space_for_bounding_sphere", "input6"),
        &input6,
        |b, input| {
            b.iter(|| {
                input.go(black_box(sphere.clone()));
            });
        },
    );

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
