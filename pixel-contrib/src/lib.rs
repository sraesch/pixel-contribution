mod error;
pub mod octahedron;
mod pixel_contribution_map;
mod progress;
mod view;

pub use error::*;
pub use pixel_contribution_map::*;
use thread_local::ThreadLocal;
pub use view::*;

use std::sync::{Arc, Mutex};

use log::info;
use rasterizer::{
    clamp, Histogram, RenderOptions, RenderStats, Renderer, RendererGeometry, Scene, StatsNode,
    StatsNodeTrait,
};
use rayon::prelude::*;

/// The options for the camera configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CameraConfig {
    /// An orthographic camera with no perspective.
    Orthographic,

    /// A perspective camera with the given field of view in y-direction in radians.
    Perspective {
        /// The field of view in y-direction in radians.
        fovy: f32,
    },
}

impl CameraConfig {
    /// Returns the angle of the camera configuration in radians.
    /// For orthographic cameras, this is 0.
    #[inline]
    pub fn angle(&self) -> f32 {
        match self {
            CameraConfig::Orthographic => 0f32,
            CameraConfig::Perspective { fovy } => *fovy,
        }
    }
}

impl ToString for CameraConfig {
    fn to_string(&self) -> String {
        match self {
            CameraConfig::Orthographic => "Orthographic".to_string(),
            CameraConfig::Perspective { fovy } => {
                format!("Perspective(fovy={} degree)", fovy.to_degrees())
            }
        }
    }
}

/// The options for the pixel contribution calculation.
pub struct PixelContributionOptions {
    /// The options for the underlying renderer.
    pub render_options: RenderOptions,

    /// The number of threads to use for the computation.
    pub num_threads: usize,

    /// The size of the quadratic pixel contribution map.
    pub contrib_map_size: usize,

    /// The camera config to be used for calculating the pixel contribution.
    pub camera_config: CameraConfig,
}

/// Computes the pixel contribution map for the given scene.
///
/// # Arguments
/// * `scene` - The scene for which the pixel contribution map should be computed.
/// * `stats` - The stats node to log the timing for the computation.
/// * `options` - The options for the pixel contribution calculation.
/// * `render_stats` - The stats node to log the rendering stats.
pub fn compute_contribution_map<R>(
    scene: &Scene,
    stats: StatsNode,
    options: &PixelContributionOptions,
    render_stats: &mut RenderStats,
) -> PixelContributionMap
where
    R: Renderer,
{
    let _t = stats.register_timing();

    // initialize thread pool for rayon if the thread pool is not already initialized
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(options.num_threads)
        .build()
        .unwrap();

    // initialize render stats to 0
    *render_stats = Default::default();

    // Determine the maximum number of pixels that can be filled. This can only be the bounding
    // sphere fit tightly into the screen, i.e., the largest possible sphere on the screen.
    // Therefore, the maximal number of possible pixels is the area of the 2D sphere filling
    // the quadratic frame.
    let max_num_pixels_filled = {
        let r = options.render_options.frame_size as f32 / 2f32;
        std::f32::consts::PI * r * r
    };

    let angle = match options.camera_config {
        CameraConfig::Orthographic => 0f32,
        CameraConfig::Perspective { fovy } => fovy,
    };

    let contrib_map_size = options.contrib_map_size;
    let render_options = options.render_options.clone();
    let descriptor = PixelContribColorMapDescriptor::new(contrib_map_size, angle);

    info!(
        "Computing pixel contribution map for {}x{} pixels",
        contrib_map_size, contrib_map_size
    );

    let geo = R::G::new(scene, stats.get_child("render_geo"));
    let bounding_sphere = scene.compute_bounding_sphere();
    info!(
        "Bounding sphere: Center={}, Radius={}",
        bounding_sphere.center, bounding_sphere.radius
    );

    let mtx_render_stats = Arc::new(Mutex::new(RenderStats::default()));
    let renderer: ThreadLocal<Arc<Mutex<R>>> = ThreadLocal::new();
    let progress = Arc::new(Mutex::new(progress::Progress::new(descriptor.num_values())));

    let pixel_contrib = thread_pool.install(|| {
        let mut pixel_contrib = PixelContributionMap::new(descriptor);

        pixel_contrib
            .pixel_contrib
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, p)| {
                progress.lock().unwrap().update();

                // create renderer if not already done
                let mut renderer = renderer
                    .get_or(|| {
                        let mut r = R::new(stats.clone());
                        r.initialize(render_options.clone()).unwrap();

                        Arc::new(Mutex::new(r))
                    })
                    .lock()
                    .unwrap();

                let camera_dir = descriptor.camera_dir_from_index(index);

                // create view based on the view direction
                let view =
                    View::new_from_sphere(&bounding_sphere, options.camera_config, camera_dir);

                // render the scene
                let renderer: &mut R = &mut renderer;
                let num_pixels_filled =
                    count_number_of_filled_pixel(renderer, &view, &geo, mtx_render_stats.clone());

                *p = num_pixels_filled as f32 / max_num_pixels_filled;
            });

        pixel_contrib
    });

    *render_stats = mtx_render_stats.lock().unwrap().clone();

    println!();

    info!(
        "Max contribution: {} ",
        pixel_contrib
            .pixel_contrib
            .iter()
            .fold(0f32, |a, b| a.max(*b))
    );

    pixel_contrib
}

/// Counts the number of filled pixels for the given view and geometry using the given renderer.
///
/// # Arguments
/// * `renderer` - The renderer to use for the computation.
/// * `view` - The view for which the number of filled pixels should be computed.
/// * `geo` - The geometry to render.
/// * `mtx_render_stats` - The stats node to log the rendering stats.
fn count_number_of_filled_pixel<R: Renderer>(
    renderer: &mut R,
    view: &View,
    geo: &R::G,
    mtx_render_stats: Arc<Mutex<RenderStats>>,
) -> u32 {
    let mut histogram = Histogram::new();
    let r = renderer.render_frame(
        geo,
        &mut histogram,
        None,
        view.view_matrix,
        view.projection_matrix,
    );

    *mtx_render_stats.lock().unwrap() += r;

    histogram.iter().sum()
}

/// The color map for encoding the pixel contribution.
pub trait ColorMap {
    /// Maps the given value to a color.
    fn map(&self, value: f64) -> (u8, u8, u8);
}

pub struct TurboColorMap {
    g: colorgrad::Gradient,
    min_val: f64,
    max_val: f64,
}

impl Default for TurboColorMap {
    fn default() -> Self {
        Self::new()
    }
}

impl TurboColorMap {
    pub fn new() -> Self {
        let g = colorgrad::turbo();
        let (min_val, max_val) = g.domain();

        Self {
            g,
            min_val,
            max_val,
        }
    }
}

impl ColorMap for TurboColorMap {
    fn map(&self, value: f64) -> (u8, u8, u8) {
        let value = clamp(
            value * self.max_val + (1f64 - value) * self.min_val,
            self.min_val,
            self.max_val,
        );
        let c = self.g.at(value).to_rgba8();

        (c[0], c[1], c[2])
    }
}

pub struct GrayScaleColorMap {}

impl Default for GrayScaleColorMap {
    fn default() -> Self {
        Self::new()
    }
}

impl GrayScaleColorMap {
    pub fn new() -> Self {
        Self {}
    }
}

impl ColorMap for GrayScaleColorMap {
    fn map(&self, value: f64) -> (u8, u8, u8) {
        let value = clamp(value * 255.0, 0.0, 255.0).round() as u8;
        (value, value, value)
    }
}
