mod cad_model;
mod geometry;
mod options;
mod sphere;

use std::error::Error;

use cad_model::CADModel;
use clap::Parser;
use log::{debug, error, info, trace, LevelFilter};
use math::extract_camera_position;
use nalgebra_glm::{Vec3, Vec4};
use options::Options;

use anyhow::Result;
use pixel_contrib::{PixelContributionMap, PixelContributionMaps};
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

    pixel_contrib: PixelContributionMap,
}

impl ViewerImpl {
    pub fn new(options: Options) -> Result<Self> {
        // load pixel contribution
        info!(
            "Loading pixel contribution maps from {}...",
            options.pixel_contribution.display()
        );
        let pixel_contrib_maps =
            PixelContributionMaps::from_file(options.pixel_contribution.as_path())?;
        info!(
            "Loading pixel contribution maps from {}...DONE",
            options.pixel_contribution.display()
        );
        Self::print_pixel_contribution_maps_info(&pixel_contrib_maps);

        let pixel_contrib = pixel_contrib_maps.get_maps().first().unwrap();

        Ok(Self {
            options,
            camera: Default::default(),
            sphere: Default::default(),
            cad_model: None,
            bounding_sphere: BoundingSphere::from((Vec3::new(0.0, 0.0, 0.0), 1.0)),
            sphere_transparency: 0.5,
            pixel_contrib: pixel_contrib.clone(),
        })
    }

    /// Prints the pixel contribution maps information to the log.
    ///
    /// # Arguments
    /// * `p` - The pixel contribution maps.
    fn print_pixel_contribution_maps_info(p: &PixelContributionMaps) {
        info!("Found {} pixel contribution maps", p.get_maps().len());

        let size = p
            .get_maps()
            .first()
            .map(|p| p.descriptor.size())
            .unwrap_or_default();
        info!("Map Size: {}", size);

        let mut supported_angles: String = String::new();
        p.get_maps().iter().for_each(|p| {
            let angle = p.descriptor.camera_angle().to_degrees();
            supported_angles = if supported_angles.is_empty() {
                format!("{}", angle)
            } else {
                format!("{}, {}", supported_angles, angle)
            };
        });

        info!("Supported Angles (in degree): {}", supported_angles);
    }
}

impl EventHandler for ViewerImpl {
    fn setup(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        info!("setup...");

        self.sphere.setup(&self.pixel_contrib)?;

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
        if self.sphere_transparency > 0.0 {
            self.sphere.render(&combined_mat, self.sphere_transparency);
        }
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
                "c" => {
                    if pressed {
                        let (w, h, values) = FrameBuffer::get_depth_buffer_values();
                        info!("Read depth buffer with size {}x{}", w, h);

                        let num_rasterized_pixels = values.iter().filter(|v| **v != 1.0).count();

                        let (model_view, fovy, height) = {
                            let data = self.camera.get_data();
                            (
                                data.get_model_matrix(),
                                data.get_fovy(),
                                data.get_window_size().1 as f32,
                            )
                        };

                        let cam_pos = match extract_camera_position(&model_view) {
                            Some(cam_pos) => {
                                debug!("Camera position: {:?}", cam_pos);
                                cam_pos
                            }
                            None => {
                                error!("Failed to extract camera position from model view matrix");
                                return;
                            }
                        };

                        let sphere_radius = estimate_bounding_sphere_radius_on_screen(
                            &cam_pos,
                            fovy,
                            &self.bounding_sphere,
                        ) * height
                            / 2.0;

                        let predicted_sphere_pixels =
                            sphere_radius * sphere_radius * std::f32::consts::PI;

                        let cam_dir =
                            nalgebra_glm::normalize(&(self.bounding_sphere.center - cam_pos));

                        let pixel_contrib_value =
                            self.pixel_contrib.get_pixel_contrib_for_camera_dir(cam_dir);

                        let num_pixels =
                            (pixel_contrib_value * predicted_sphere_pixels).round() as usize;

                        let estimated_cam_angle =
                            estimate_camera_angle(&cam_pos, &self.bounding_sphere);

                        info!(" -- Prediction --");
                        info!("Camera direction: {:?}", cam_dir);
                        info!("Bounding sphere radius on screen: {}", sphere_radius);
                        info!(
                            "Pixel contribution factor from map: {}",
                            pixel_contrib_value
                        );
                        info!(
                            "Estimated camera angle: {}",
                            estimated_cam_angle.to_degrees()
                        );

                        info!(
                            "Number of actually rasterized pixels (Framebuffer): {}",
                            num_rasterized_pixels
                        );
                        info!(
                            "Predicted number of filled pixels (Bounding Sphere): {}",
                            predicted_sphere_pixels
                        );
                        info!("Predicted number of filled pixels: {}", num_pixels);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Estimates the radius of the bounding sphere on the screen in the range [0, 1].
/// A value of 1 means that the sphere fills the screen completely.
/// Note: This does not take the aspect ratio or the frustum into account.
///
/// # Arguments
/// * `cam_pos` - The position of the camera.
/// * `fovy` - The field of view in y-direction in radians.
/// * `sphere` - The bounding sphere.
fn estimate_bounding_sphere_radius_on_screen(
    cam_pos: &Vec3,
    fovy: f32,
    sphere: &BoundingSphere,
) -> f32 {
    let d = nalgebra_glm::distance(cam_pos, &sphere.center);

    // project the ray that tangentially touches the sphere onto the plane that is 'd' units away
    // from the camera
    let phi = (sphere.radius / d).asin();
    let projected_radius = phi.tan() * d;

    // now compute half the length of the side of the frustum at the distance 'd'
    let r_capital = (fovy / 2.0).tan() * d;

    // use this radius to estimate how much the screen is being filled by the sphere
    projected_radius / r_capital
}

/// Estimates the angle of the camera based on the bounding sphere.
///
/// # Arguments
/// * `cam_pos` - The position of the camera.
/// * `sphere` - The bounding sphere.
fn estimate_camera_angle(cam_pos: &Vec3, sphere: &BoundingSphere) -> f32 {
    let d = nalgebra_glm::distance(cam_pos, &sphere.center);
    (sphere.radius / d).atan() * 2f32
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

    let viewer = ViewerImpl::new(options)?;
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
