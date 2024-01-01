mod error;
pub mod octahedron;
mod sphere;
mod view;

pub use error::*;
pub use sphere::*;
pub use view::*;

use std::path::Path;

use image::GrayImage;
use log::info;
use nalgebra_glm::Vec2;
use rasterizer::{
    clamp, Histogram, RenderOptions, RenderStats, Renderer, RendererGeometry, Scene, StatsNode,
    StatsNodeTrait,
};

use crate::octahedron::decode_octahedron_normal;

/// The options for the pixel contribution calculation.
pub struct PixelContributionOptions {
    /// The options for the underlying renderer.
    pub render_options: RenderOptions,

    /// The size of the quadratic pixel contribution map.
    pub contrib_map_size: usize,

    /// The field of view in y-direction in radians.
    pub fovy: f32,
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
) -> PixelContribution
where
    R: Renderer,
{
    let _t = stats.register_timing();

    *render_stats = Default::default();

    let total_num_pixels =
        (options.render_options.frame_size * options.render_options.frame_size) as f32;
    let contrib_map_size = options.contrib_map_size;
    let render_options = options.render_options.clone();

    info!(
        "Computing pixel contribution map for {}x{} pixels",
        contrib_map_size, contrib_map_size
    );

    let geo = R::G::new(scene, stats.get_child("render_geo"));
    let bounding_sphere = compute_bounding_sphere(scene);
    info!(
        "Bounding sphere: Center={}, Radius={}",
        bounding_sphere.center, bounding_sphere.radius
    );

    let mut renderer = R::new(stats.clone());
    renderer.initialize(render_options).unwrap();

    let mut pixel_contrib = PixelContribution::new(contrib_map_size);

    pixel_contrib
        .pixel_contrib
        .iter_mut()
        .enumerate()
        .for_each(|(index, p)| {
            print_progress(index, contrib_map_size * contrib_map_size);

            // compute normalized pixel position
            let (x, y) = (
                (index % contrib_map_size) as f32,
                (index / contrib_map_size) as f32,
            );

            let (u, v) = (
                (x + 0.5) / contrib_map_size as f32,
                (y + 0.5) / contrib_map_size as f32,
            );

            // determine the view direction
            let dir = decode_octahedron_normal(Vec2::new(u, v));

            // create view for the current pixel
            let fovy = options.fovy;
            let view = View::new_from_sphere(&bounding_sphere, fovy, dir);

            // render the scene
            let mut histogram = Histogram::new();
            *render_stats += renderer.render_frame(
                &geo,
                &mut histogram,
                None,
                view.view_matrix,
                view.projection_matrix,
            );

            let total_num_pixel: u32 = histogram.iter().sum();
            *p = total_num_pixel as f32 / total_num_pixels;
        });

    pixel_contrib
}

/// The resulting pixel contribution for all possible views.
#[derive(Clone)]
pub struct PixelContribution {
    /// The size of the quadratic pixel contribution map.
    pub size: usize,

    /// The 2D map for the pixel contribution of each view. Each pixel position represents a view.
    /// The normalized pixel position (u,v) is mapped to a normal using octahedral projection.
    /// The normal then defines the camera view direction.
    /// The values are in the range [0, 1].
    pub pixel_contrib: Vec<f32>,
}

impl PixelContribution {
    /// Creates a new pixel contribution map with the given size.
    ///
    /// # Arguments
    /// * `size` - The size of the quadratic pixel contribution map.
    pub fn new(size: usize) -> Self {
        Self {
            size,
            pixel_contrib: vec![0.0; size * size],
        }
    }

    /// Writes the pixel contribution map to the given path as image.
    ///
    /// # Arguments
    /// * `path` - The path to which the image should be written.
    pub fn write_image<P: AsRef<Path>>(&self, path: P) -> error::Result<()> {
        let mut img = GrayImage::new(self.size as u32, self.size as u32);

        self.pixel_contrib
            .iter()
            .zip(img.pixels_mut())
            .for_each(|(p, pixel)| {
                let v = clamp((p * 255.0).round(), 0f32, 255f32) as u8;
                pixel[0] = v;
            });

        img.save(path)?;

        Ok(())
    }
}

/// Computes the bounding sphere of the given scene.
///
/// # Arguments
/// * `scene` - The scene for which the bounding sphere should be computed.
fn compute_bounding_sphere(scene: &Scene) -> Sphere {
    let sphere = scene.compute_bounding_sphere();

    Sphere::from(sphere)
}

/// Prints the progress of the current task to the console.
///
/// # Arguments
/// * `cur` - The current progress.
/// * `total` - The total number of steps.
fn print_progress(cur: usize, total: usize) {
    let bar_length = 50;
    let progress = cur as f32 / total as f32;
    let num_bars = (progress * bar_length as f32) as usize;
    let num_spaces = bar_length - num_bars;

    print!("\r[");
    for _ in 0..num_bars {
        print!("=");
    }
    for _ in 0..num_spaces {
        print!(" ");
    }
    print!("] {:.2}%", progress * 100.0);
}
