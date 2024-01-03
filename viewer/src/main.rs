mod cad_model;
mod geometry;
mod options;
mod sphere;

use std::error::Error;

use cad_model::CADModel;
use clap::Parser;
use log::{debug, error, info, trace, LevelFilter};
use nalgebra_glm::{Vec3, Vec4};
use options::Options;

use anyhow::Result;
use rasterizer::BoundingSphere;
use render_lib::{
    camera::Camera, create_and_run_canvas, CanvasOptions, EventHandler, FrameBuffer, Key,
    MouseButton,
};
use sphere::Sphere;

struct ViewerImpl {
    options: Options,
    camera: Camera,

    sphere: Sphere,
    cad_model: Option<CADModel>,
}

impl ViewerImpl {
    pub fn new(options: Options) -> Self {
        Self {
            options,
            camera: Default::default(),
            sphere: Default::default(),
            cad_model: None,
        }
    }
}

impl EventHandler for ViewerImpl {
    fn setup(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        info!("setup...");

        self.sphere.setup(self.options.image_file.as_path())?;

        self.cad_model = match CADModel::new(&self.options.model_file) {
            Ok(cad_model) => Some(cad_model),
            Err(err) => {
                error!("Failed to load CAD model: {}", err);
                None
            }
        };

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

        self.sphere.render(&combined_mat);
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
