use log::info;
use nalgebra_glm::{Mat4, Vec3};

use crate::{
    math::transform_vec3,
    scene::{CompressedPositions, CompressedPositionsRaw, IntegerTrait, Scene},
    spatial::simple::compute_sorting,
    stats::StatsNode,
    RendererGeometry, Result, StatsNodeTrait,
};

use super::{
    frame_buffer::{DepthBuffer, FrameBuffer, SimpleDepthBuffer},
    Frame, Histogram, Page, RenderOptions, RenderStats, Renderer,
};

/// A very simple single-threaded rasterizer without any acceleration structures.
pub struct SimpleRasterizer {
    stats: StatsNode,
    frame_buffer: FrameBuffer<SimpleDepthBuffer>,
}

impl Renderer for SimpleRasterizer {
    type G = SimpleRasterizerGeometry;

    fn new(stats: StatsNode) -> Self {
        Self {
            stats,
            frame_buffer: Default::default(),
        }
    }

    fn get_name(&self) -> &str {
        "Simple Rasterizer"
    }

    fn initialize(&mut self, options: RenderOptions) -> Result<()> {
        info!(
            "Initialize simple rasterizer with size {}x{}",
            options.frame_size, options.frame_size
        );

        self.frame_buffer = FrameBuffer::new(options.frame_size);

        Ok(())
    }

    fn render_frame(
        &mut self,
        geo: &SimpleRasterizerGeometry,
        histogram: &mut Histogram,
        frame: Option<&mut Frame>,
        view_matrix: nalgebra_glm::Mat4,
        projection_matrix: nalgebra_glm::Mat4,
    ) -> Result<RenderStats> {
        let _t = self.stats.register_timing();
        let mut stats: RenderStats = Default::default();

        let pages = geo.pages.as_slice();

        let frame_buffer = &mut self.frame_buffer;

        frame_buffer.clear();

        // compute matrix for projection
        let pmmat = projection_matrix * view_matrix;

        // rasterize all triangles in all pages
        pages.iter().for_each(|page| {
            frame_buffer.rasterize_page(page, &pmmat);
            stats.num_triangles += page.triangles.len();
        });

        // compute resulting histogram
        self.frame_buffer.compute_histogram(histogram);

        // Check if we've to return the frame buffer
        if let Some(f) = frame {
            self.frame_buffer.get_frame(f);
        }

        Ok(stats)
    }
}

trait PageRasterizer {
    /// Dequantize and rasterize the given page.
    ///
    /// # Arguments
    /// * `page` - the page to rasterize.
    /// * `positions` - The positions values of the given page.
    /// * `pmmat` - The combined project and model-view matrix.
    fn rasterize_and_dequantize_page<Integer: IntegerTrait>(
        &mut self,
        page: &Page,
        positions: &CompressedPositionsRaw<Integer>,
        pmmat: &Mat4,
    );

    /// Rasterizes the given page into the internal frame buffer.
    ///
    /// # Arguments
    /// * `page` - The page to rasterize.
    /// * `pmmat` - The combined projection, view and model matrix.
    fn rasterize_page(&mut self, page: &Page, pmmat: &Mat4) {
        match &page.position {
            CompressedPositions::Bit8(c) => self.rasterize_and_dequantize_page(page, c, pmmat),
            CompressedPositions::Bit16(c) => self.rasterize_and_dequantize_page(page, c, pmmat),
            CompressedPositions::Bit32(c) => self.rasterize_and_dequantize_page(page, c, pmmat),
        }
    }
}

impl<D: DepthBuffer> PageRasterizer for FrameBuffer<D> {
    /// Dequantizes the given page and rasterizes it into the frame buffer.
    ///
    /// # Arguments
    /// * `page` - the page to rasterize.
    /// * `positions` - The positions values of the given page.
    /// * `pmmat` - The combined project and model-view matrix.
    fn rasterize_and_dequantize_page<Integer: IntegerTrait>(
        &mut self,
        page: &Page,
        positions: &CompressedPositionsRaw<Integer>,
        pmmat: &Mat4,
    ) {
        let triangles = &page.triangles;
        let local_ids = &page.local_object_ids;
        let object_id_map = &page.object_id_map;

        let size = self.get_frame_size() as f32;

        let m = pmmat
            * positions
                .get_quantization_operator()
                .get_descriptor()
                .get_dequantization_matrix();

        for (triangle, local_id) in triangles.iter().zip(local_ids.iter()) {
            let id = object_id_map[*local_id as usize];

            let p0 = positions.get_position_normalized(triangle[0] as usize);
            let p1 = positions.get_position_normalized(triangle[1] as usize);
            let p2 = positions.get_position_normalized(triangle[2] as usize);

            let p0 = project_pos(size, &m, &p0);
            let p1 = project_pos(size, &m, &p1);
            let p2 = project_pos(size, &m, &p2);

            self.rasterize(id, &p0, &p1, &p2);
        }
    }
}

pub struct SimpleRasterizerGeometry {
    pub pages: Vec<Page>,
}

impl RendererGeometry for SimpleRasterizerGeometry {
    fn new(scene: &Scene, stats: StatsNode) -> Self {
        let pages = compute_sorting(scene, stats);

        Self { pages }
    }
}

/// Transforms the given position from world coordinates into screen coordinates.
///
/// # Arguments
/// * `size` - The size of the quadratic frame in pixels.
/// * `t` - The combined projection, view and model matrix.
/// * `p` - The position in world coordinates to project.
#[inline]
pub fn project_pos(size: f32, t: &Mat4, p: &Vec3) -> Vec3 {
    let p = transform_vec3(t, p);

    let x = (p[0] * 0.5 + 0.5) * size;
    let y = (p[1] * 0.5 + 0.5) * size;
    let z = (1.0 + p[2]) * 0.5;

    Vec3::new(x, y, z)
}
