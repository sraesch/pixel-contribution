mod geometry;
mod options;

use std::{error::Error, mem::size_of};

use clap::Parser;
use log::{debug, error, info, trace, LevelFilter};
use nalgebra_glm::{Vec3, Vec4};
use options::Options;

use anyhow::Result;
use rasterizer::BoundingSphere;
use render_lib::{
    camera::Camera, create_and_run_canvas, Attribute, AttributeBlock, Bind, CanvasOptions,
    DataType, DrawCall, EventHandler, Filtering, FrameBuffer, GPUBuffer, GPUBufferType, IndexData,
    Key, MouseButton, PrimitiveType, Shader, Texture, Uniform,
};

use crate::geometry::create_sphere;

struct ViewerImpl {
    options: Options,
    camera: Camera,

    texture: Texture,

    shader: Shader,

    uniform_texture: Uniform,
    uniform_combined_mat: Uniform,

    positions: GPUBuffer,
    indices: GPUBuffer,
    num_indices: usize,
    draw_call: DrawCall,
}

impl ViewerImpl {
    pub fn new(options: Options) -> Self {
        Self {
            options,
            camera: Default::default(),
            texture: Default::default(),
            shader: Default::default(),
            uniform_texture: Default::default(),
            uniform_combined_mat: Default::default(),
            positions: GPUBuffer::new(GPUBufferType::Vertices),
            indices: GPUBuffer::new(GPUBufferType::Indices),
            num_indices: 0,
            draw_call: Default::default(),
        }
    }
}

impl EventHandler for ViewerImpl {
    fn setup(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        info!("setup...");

        let vert_shader = include_str!("../shader/simple.vert");
        let frag_shader = include_str!("../shader/simple.frag");

        info!("compile shader...");
        self.shader.load(vert_shader, frag_shader)?;
        self.uniform_texture = self.shader.get_uniform("uniform_texture").unwrap();
        self.uniform_combined_mat = self.shader.get_uniform("uniform_combined_mat").unwrap();
        info!("compile shader...DONE");

        // initialize texture
        self.texture
            .generate_from_image(&self.options.image_file, Filtering::Linear)?;

        // initializes quad geometry
        let sphere_geo = create_sphere(1.0, 100, 100);
        self.positions.set_data(&sphere_geo.0);
        self.indices.set_data(&sphere_geo.1);
        self.num_indices = sphere_geo.1.len();

        // create vertex array
        self.draw_call.set_data(&[AttributeBlock {
            vertex_data: &self.positions,
            attributes: vec![Attribute {
                offset: 0,
                stride: size_of::<f32>() * 3,
                num_components: 3,
                data_type: DataType::Float,
                is_integer: false,
                normalized: false,
            }],
        }]);

        FrameBuffer::depthtest(true);

        self.camera.update_window_size(width, height);
        self.camera
            .focus(&BoundingSphere::from((Vec3::new(0.0, 0.0, 0.0), 2.0)))
            .unwrap();

        info!("setup...DONE");

        Ok(())
    }

    fn stop(&mut self) {
        info!("stop");
    }

    fn next_frame(&mut self) {
        trace!("debug frame");
        FrameBuffer::clear_buffers(&Vec4::new(0.0, 0.1, 0.2, 1.0));

        let combined_mat = self.camera.get_data().get_combined_matrix();
        self.uniform_combined_mat.set_matrix4(&combined_mat);

        self.shader.bind();
        self.texture.bind();
        self.uniform_texture.set_int(0);
        self.draw_call.draw_with_indices(
            PrimitiveType::Triangles,
            &self.indices,
            &IndexData {
                datatype: DataType::UnsignedInt,
                offset: 0,
                num: self.num_indices,
            },
        );
    }

    fn resize(&mut self, w: u32, h: u32) {
        debug!("resize({}, {})", w, h);

        self.camera.update_window_size(w, h);
    }

    fn cursor_move(&mut self, x: f64, y: f64) {
        trace!("cursor_move({}, {})", x, y);

        self.camera.event_mouse_motion(x, y);
    }

    fn mouse_button(&mut self, x: f64, y: f64, button: MouseButton, pressed: bool) {
        trace!("mouse_button({}, {}, {:?}, {})", x, y, button, pressed);

        self.camera.event_mouse_button(x, y, button, pressed);
    }

    fn keyboard_event(&mut self, key: Key, pressed: bool) {
        trace!("keyboard_event({:?}, {})", key, pressed);
    }
}

/// Parses the program arguments and returns None, if no arguments were provided and Some otherwise.
fn parse_args() -> Result<Options> {
    let options = Options::parse();
    Ok(options)
}

/// Initializes the program logging
///
/// # Arguments
/// * `filter` - The log level filter.
fn initialize_logging(filter: LevelFilter) {
    simple_logging::log_to(std::io::stdout(), filter);
}

/// Runs the viewer program.
fn run_program() -> Result<()> {
    let options = parse_args()?;
    initialize_logging(options.log_level.into());

    options.dump_to_log();

    let viewer = ViewerImpl::new(options);
    create_and_run_canvas(
        CanvasOptions {
            title: "Viewer".to_string(),
            width: 800,
            height: 600,
        },
        viewer,
    )?;

    Ok(())
}

fn main() {
    match run_program() {
        Ok(()) => {
            info!("SUCCESS");
        }
        Err(err) => {
            error!("Error: {}", err);
            error!("FAILED");

            std::process::exit(-1);
        }
    }
}
