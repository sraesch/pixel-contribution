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
    camera::Camera, configure_culling, create_and_run_canvas, BlendFactor, CanvasOptions,
    EventHandler, FaceCulling, FrameBuffer, Key, MouseButton,
};
use sphere::Sphere;

struct ViewerImpl {
    options: Options,
    camera: Camera,

    sphere: Sphere,
    cad_model: Option<CADModel>,
    bounding_sphere: BoundingSphere,

    sphere_transparency: f32,
}

impl ViewerImpl {
    pub fn new(options: Options) -> Self {
        Self {
            options,
            camera: Default::default(),
            sphere: Default::default(),
            cad_model: None,
            bounding_sphere: BoundingSphere::from((Vec3::new(0.0, 0.0, 0.0), 1.0)),
            sphere_transparency: 0.5,
        }
    }
}

impl EventHandler for ViewerImpl {
    fn setup(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        info!("setup...");

        self.sphere.setup(self.options.image_file.as_path())?;

        self.cad_model = match CADModel::new(&self.options.model_file) {
            Ok(cad_model) => {
                self.bounding_sphere = cad_model.get_bounding_sphere().clone();

                info!("CAD-Data Bounding sphere: {:?}", self.bounding_sphere);

                self.camera.focus(&self.bounding_sphere).unwrap();
                Some(cad_model)
            }
            Err(err) => {
                error!("Failed to load CAD model: {}", err);

                None
            }
        };

        FrameBuffer::depthtest(true);
        configure_culling(FaceCulling::None);

        self.camera.update_window_size(width, height);
        info!("setup...DONE");

        Ok(())
    }

    fn stop(&mut self) {
        info!("stop");
    }

    fn next_frame(&mut self) {
        trace!("debug frame");
        FrameBuffer::clear_buffers(&Vec4::new(0.0, 0.1, 0.2, 1.0));

        let model_view_mat = self.camera.get_data().get_model_matrix();
        let proj_mat = self.camera.get_data().get_projection_matrix();

        if let Some(cad_model) = &self.cad_model {
            cad_model.render(&model_view_mat, &proj_mat);
        }

        // create the translation and scale matrix for the sphere based on the bounding sphere
        let sphere_mat = {
            let scale_mat = nalgebra_glm::scaling(&Vec3::new(
                self.bounding_sphere.radius,
                self.bounding_sphere.radius,
                self.bounding_sphere.radius,
            ));
            let translation_mat = nalgebra_glm::translation(&self.bounding_sphere.center);

            translation_mat * scale_mat
        };

        let combined_mat = self.camera.get_data().get_combined_matrix() * sphere_mat;

        FrameBuffer::set_blending(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        configure_culling(FaceCulling::Back);
        self.sphere.render(&combined_mat, self.sphere_transparency);
        configure_culling(FaceCulling::None);
        FrameBuffer::disable_blend();
    }

    fn resize(&mut self, w: u32, h: u32) {
        debug!("resize({}, {})", w, h);

        FrameBuffer::viewport(0, 0, w, h);
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

        if let Key::Character(c) = key {
            match c.as_str() {
                "w" => {
                    if pressed {
                        self.sphere_transparency = (self.sphere_transparency + 0.1).min(1.0);
                    }
                }
                "s" => {
                    if pressed {
                        self.sphere_transparency = (self.sphere_transparency - 0.1).max(0.0);
                    }
                }
                _ => {}
            }
        }
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
    env_logger::builder().filter_level(filter).init();
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
