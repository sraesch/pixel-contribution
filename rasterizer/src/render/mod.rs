use crate::{scene::Scene, stats::StatsNode, Result};

mod frame;
pub mod frame_buffer;
mod page;
pub mod simple_rasterizer;

pub use frame::*;
pub use page::*;

use nalgebra_glm::Mat4;

/// A histogram of the object ids in the frame buffer. The index of the vector is the object id
/// and its value is the number of pixels with that object id.
pub type Histogram = Vec<u32>;

/// The options for a renderer
#[derive(Clone)]
pub struct RenderOptions {
    /// The number of threads to be used for the renderer
    pub num_threads: usize,

    /// The size of the quadratic frame buffer
    pub frame_size: usize,
}

/// Resulting stats about the rendering process
#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    /// The number of triangles processed, i.e., that could not be avoided through acceleration
    /// structures or other means.
    pub num_triangles: usize,
}

impl std::ops::Add<Self> for RenderStats {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            num_triangles: self.num_triangles + rhs.num_triangles,
        }
    }
}

impl std::ops::AddAssign<Self> for RenderStats {
    fn add_assign(&mut self, rhs: Self) {
        self.num_triangles += rhs.num_triangles;
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            num_threads: 1,
            frame_size: 512,
        }
    }
}

/// The geometry used for rendering.
pub trait RendererGeometry {
    /// Creates a new renderer geometry from the given scene and takes ownership of the scene.
    ///
    /// # Arguments
    /// * `scene` - The scene to create the renderer geometry from.
    /// * `stats` - The stats node to log the timing for creating the geometry.
    fn new(scene: &Scene, stats: StatsNode) -> Self;
}

/// A renderer renders a single frame based on the provided scene.
/// The resulting frame contains the object for each pixel.
pub trait Renderer {
    /// The optimized geometry used for rendering.
    type G: RendererGeometry;

    /// Creates and returns a new renderer instance.
    ///
    /// # Arguments
    /// * `stats` - The stats node into which the renderer registers all its times.
    fn new(stats: StatsNode) -> Self;

    /// Returns the name of the renderer
    fn get_name(&self) -> &str;

    /// Initializes the renderer with the given frame size.
    ///
    /// # Arguments
    /// * `options` - The renderer options.
    fn initialize(&mut self, options: RenderOptions) -> Result<()>;

    /// Renders a frame determines the visible ids of the objects.
    ///
    /// # Arguments
    /// * `geo` - The geometry to render.
    /// * `histogram` - A mutable reference for returning the object id histogram.
    /// * `frame` - Optionally a mutable reference onto the frame to return the resulting pixels.
    /// * `view_matrix` - The camera view matrix.
    /// * `projection_matrix` - The camera projection matrix.
    fn render_frame(
        &mut self,
        geo: &Self::G,
        histogram: &mut Histogram,
        frame: Option<&mut Frame>,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) -> RenderStats;
}
