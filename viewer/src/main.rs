mod options;

use std::{error::Error, mem::size_of};

use clap::Parser;
use log::{error, info, trace, LevelFilter};
use nalgebra_glm::Vec4;
use options::Options;

use anyhow::Result;
use render_lib::{
    create_and_run_canvas, Attribute, AttributeBlock, Bind, CanvasOptions, DataType, DrawCall,
    EventHandler, Filtering, FrameBuffer, GPUBuffer, GPUBufferType, Key, MouseButton,
    PrimitiveType, Shader, Texture, Uniform,
};

struct ViewerImpl {
    options: Options,

    texture: Texture,

    shader: Shader,

    uniform_texture: Uniform,

    positions: GPUBuffer,
    draw_call: DrawCall,
}

impl ViewerImpl {
    pub fn new(options: Options) -> Self {
        Self {
            options,
            texture: Default::default(),
            shader: Default::default(),
            uniform_texture: Default::default(),
            positions: GPUBuffer::new(GPUBufferType::Vertices),
            draw_call: Default::default(),
        }
    }
}

impl EventHandler for ViewerImpl {
    fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        info!("setup...");

        let vert_shader = include_str!("../shader/simple.vert");
        let frag_shader = include_str!("../shader/simple.frag");

        info!("compile shader...");
        self.shader.load(vert_shader, frag_shader)?;
        self.uniform_texture = self.shader.get_uniform("uniform_texture").unwrap();
        info!("compile shader...DONE");

        // initialize texture
        self.texture
            .generate_from_image(&self.options.image_file, Filtering::Linear)?;

        // initializes quad geometry
        let positions: [f32; 8] = [1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32];
        self.positions.set_data(&positions);

        // create vertex array
        self.draw_call.set_data(&[AttributeBlock {
            vertex_data: &self.positions,
            attributes: vec![Attribute {
                offset: 0,
                stride: size_of::<f32>() * 2,
                num_components: 2,
                data_type: DataType::Float,
                is_integer: false,
                normalized: false,
            }],
        }]);

        info!("setup...DONE");

        Ok(())
    }

    fn stop(&mut self) {
        info!("stop");
    }

    fn next_frame(&mut self) {
        trace!("debug frame");
        FrameBuffer::clear_buffers(&Vec4::new(0.0, 0.1, 0.2, 1.0));
        FrameBuffer::depthtest(false);
        self.shader.bind();
        self.texture.bind();
        self.uniform_texture.set_int(0);
        self.draw_call
            .draw_no_indices(PrimitiveType::TriangleStrip, 0, 4);
        FrameBuffer::depthtest(true);
    }

    fn resize(&mut self, w: u32, h: u32) {
        info!("resize({}, {})", w, h);
    }

    fn cursor_move(&mut self, x: f64, y: f64) {
        trace!("cursor_move({}, {})", x, y);
    }

    fn mouse_button(&mut self, x: f64, y: f64, button: MouseButton, pressed: bool) {
        trace!("mouse_button({}, {}, {:?}, {})", x, y, button, pressed);
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
