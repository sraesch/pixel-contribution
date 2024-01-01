use cad_import::loader::{loader_gltf::LoaderGLTF, Loader, MemoryResource};
use nalgebra_glm::Mat4;
use rasterizer::{Histogram, RenderOptions, Renderer, RendererGeometry, Scene, Stats};

/// A single view with expected resulting rasterization result
pub struct TestView {
    /// The combined model and view matrix for the test
    model_view_matrix: Mat4,

    /// The projection matrix for the view
    projection_matrix: Mat4,

    /// The resulting histogram, i.e., histogram[object-id] = relative coverage
    expected_histogram: Vec<f32>,
}

/// A single test-scenario to be tested, i.e., loaded scene + expected result.
pub struct TestScenario {
    /// The scene of the test-scenario to test
    pub scene: Scene,

    pub views: Vec<TestView>,
}

/// Loads the given GLB-file.
///
/// # Arguments
/// * `data` - The GLB as binary memory block.
fn load_glb(data: &'static [u8]) -> Scene {
    let memory_resource = MemoryResource::new(data, "model/gltf-binary".to_owned());
    let gltf_loader = LoaderGLTF::new();
    let cad_data = gltf_loader.read(&memory_resource).unwrap();
    Scene::new_from_cad(&cad_data).unwrap()
}

/// Returns the two cubes test-scenario.
pub fn load_two_cubes() -> TestScenario {
    let scene_data = include_bytes!("../../../test_data/models/2Boxes.glb");
    let scene = load_glb(scene_data);

    let model_view_matrix = Mat4::from_column_slice(&[
        1f32,
        0f32,
        0f32,
        0f32,
        0f32,
        1f32,
        0f32,
        0f32,
        0f32,
        0f32,
        1f32,
        0f32,
        0f32,
        0f32,
        -7.794229f32,
        1f32,
    ]);
    let projection_matrix = Mat4::from_column_slice(&[
        1.3728658f32,
        0f32,
        0f32,
        0f32,
        0f32,
        1.8304877,
        0f32,
        0f32,
        0f32,
        0f32,
        -2.6000001f32,
        -1f32,
        0f32,
        0f32,
        -18.70615f32,
        0f32,
    ]);

    let view0 = TestView {
        model_view_matrix,
        projection_matrix,
        expected_histogram: vec![0.01382f32, 0.01382f32],
    };

    let model_view_matrix = Mat4::from_column_slice(&[
        -0.027596472,
        0.0,
        0.9996192,
        0.0,
        0.0021354153,
        0.99999774,
        5.8952377e-5,
        0.0,
        -0.9996169,
        0.0021362288,
        -0.027596409,
        0.0,
        -7.450581e-9,
        0.0,
        -3.4820378,
        1.0,
    ]);
    let projection_matrix = Mat4::from_column_slice(&[
        1.3728658, 0.0, 0.0, 0.0, 0.0, 1.8304877, 0.0, 0.0, 0.0, 0.0, -1.2721896, -1.0, 0.0, 0.0,
        -2.0085285, 0.0,
    ]);

    let view1 = TestView {
        model_view_matrix,
        projection_matrix,
        expected_histogram: vec![0f32, 0.64898f32],
    };

    let views = vec![view0, view1];

    TestScenario { scene, views }
}

/// Determines the maximal deviation of all coverage
fn max_deviation_histograms(
    ground_truth: &[f32],
    result: &Histogram,
    total_num_pixels: f32,
) -> f32 {
    assert_eq!(ground_truth.len(), result.len());

    let e = ground_truth
        .iter()
        .zip(result.iter())
        .map(|(g, r)| ((*r as f32) / total_num_pixels - *g).abs())
        .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
        .unwrap_or(0f32);

    e
}

/// Executes a test with the defined renderer and given test-scenario.
///
/// # Arguments
/// * `scenario` - The test-scenario to execute.
pub fn test_renderer<R: Renderer>(scenario: TestScenario) {
    let stats = Stats::root();

    let scene = scenario.scene;
    let views = scenario.views;

    let options = RenderOptions {
        frame_size: 256,
        ..Default::default()
    };

    let total_num_pixels = (options.frame_size * options.frame_size) as f32;

    let geo = R::G::new(&scene, stats.clone());

    let mut renderer = R::new(stats);
    renderer.initialize(options).unwrap();

    for view in views {
        let ground_truth = &view.expected_histogram;
        let mut histogram = Histogram::new();
        let model_view_matrix = view.model_view_matrix;
        let projection_matrix = view.projection_matrix;

        renderer.render_frame(
            &geo,
            &mut histogram,
            None,
            model_view_matrix,
            projection_matrix,
        );

        let error = max_deviation_histograms(ground_truth, &histogram, total_num_pixels);
        println!("Max Error: {}", error);

        assert!(error <= 0.003f32);
    }
}
