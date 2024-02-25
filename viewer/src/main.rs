mod cad_model;
mod geometry;
mod logging;
mod options;
mod screenspace;
mod sphere;

use std::error::Error;

use cad_model::CADModel;
use clap::Parser;
use log::{debug, error, info, trace};
use math::extract_camera_position;
use nalgebra_glm::{Vec3, Vec4};
use options::Options;

use anyhow::Result;
use pixel_contrib::PixelContributionMaps;
use rasterizer::BoundingSphere;
use render_lib::{
    camera::Camera, configure_culling, create_and_run_canvas, BlendFactor, CanvasOptions,
    EventHandler, FaceCulling, FrameBuffer, Key, MouseButton, NamedKey,
};
use sphere::Sphere;

use crate::screenspace::estimate_screenspace_for_bounding_sphere;

struct ViewerImpl {
    options: Options,
    camera: Camera,

    sphere: Sphere,
    cad_model: Option<CADModel>,
    bounding_sphere: BoundingSphere,

    sphere_transparency: f32,
    current_contrib_map_index: usize,

    pixel_contrib_maps: PixelContributionMaps,
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

        let mut camera = Camera::default();
        camera.get_data_mut().set_fovy(options.fovy.to_radians());

        Ok(Self {
            options,
            camera,
            sphere: Default::default(),
            cad_model: None,
            bounding_sphere: BoundingSphere::from((Vec3::new(0.0, 0.0, 0.0), 1.0)),
            sphere_transparency: 0.5,
            current_contrib_map_index: 0,
            pixel_contrib_maps,
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

        self.sphere.setup(&self.pixel_contrib_maps)?;

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
            self.sphere.render(
                &combined_mat,
                self.sphere_transparency,
                self.current_contrib_map_index,
            );
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

        if let Key::Named(name) = key {
            match name {
                NamedKey::ArrowLeft => {
                    if pressed {
                        let mut cam_pos = *self.camera.get_data_mut().get_center();
                        cam_pos[0] -= 0.1;
                        self.camera.get_data_mut().set_center(&cam_pos);
                    }
                }
                NamedKey::ArrowRight => {
                    if pressed {
                        let mut cam_pos = *self.camera.get_data_mut().get_center();
                        cam_pos[0] += 0.1;
                        self.camera.get_data_mut().set_center(&cam_pos);
                    }
                }
                NamedKey::ArrowUp => {
                    if pressed {
                        let mut cam_pos = *self.camera.get_data_mut().get_center();
                        cam_pos[1] -= 0.1;
                        self.camera.get_data_mut().set_center(&cam_pos);
                    }
                }
                NamedKey::ArrowDown => {
                    if pressed {
                        let mut cam_pos = *self.camera.get_data_mut().get_center();
                        cam_pos[1] += 0.1;
                        self.camera.get_data_mut().set_center(&cam_pos);
                    }
                }
                _ => {}
            }
        }

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
                "m" => {
                    if pressed {
                        self.current_contrib_map_index = (self.current_contrib_map_index + 1)
                            % self.pixel_contrib_maps.get_maps().len();

                        let contrib_map =
                            &self.pixel_contrib_maps.get_maps()[self.current_contrib_map_index];
                        info!(
                            "Switched to pixel contribution map (i={}, angle={:?})",
                            self.current_contrib_map_index,
                            contrib_map.descriptor.camera_angle().to_degrees()
                        );
                    }
                }
                "c" => {
                    if pressed {
                        let (w, h, values) = FrameBuffer::get_depth_buffer_values();
                        info!("Read depth buffer with size {}x{}", w, h);

                        let num_rasterized_pixels = values.iter().filter(|v| **v != 1.0).count();

                        let (model_view, projection_matrix, height) = {
                            let data = self.camera.get_data();
                            (
                                data.get_model_matrix(),
                                data.get_projection_matrix(),
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

                        let predicted_sphere_pixels = estimate_screenspace_for_bounding_sphere(
                            &model_view,
                            &projection_matrix,
                            self.bounding_sphere.clone(),
                            height,
                        )
                        .unwrap();

                        let cam_dir =
                            nalgebra_glm::normalize(&(self.bounding_sphere.center - cam_pos));

                        let pixel_contrib =
                            &self.pixel_contrib_maps.get_maps()[self.current_contrib_map_index];
                        let pixel_contrib_value =
                            pixel_contrib.get_pixel_contrib_for_camera_dir(cam_dir);

                        let num_pixels =
                            (pixel_contrib_value * predicted_sphere_pixels).round() as usize;

                        let estimated_cam_angle =
                            estimate_camera_angle(&cam_pos, &self.bounding_sphere);

                        info!(" -- Prediction --");
                        info!("Camera direction: {:?}", cam_dir);
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
                        info!(
                            "Error: {}%",
                            (num_pixels as f32 / num_rasterized_pixels as f32 - 1f32) * 100.0
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

/// Estimates the angle of the camera based on the bounding sphere.
///
/// # Arguments
/// * `cam_pos` - The position of the camera.
/// * `sphere` - The bounding sphere.
fn estimate_camera_angle(cam_pos: &Vec3, sphere: &BoundingSphere) -> f32 {
    let d = nalgebra_glm::distance(cam_pos, &sphere.center);
    (sphere.radius / d).asin() * 2f32
}

/// Parses the program arguments and returns None, if no arguments were provided and Some otherwise.
fn parse_args() -> Result<Options> {
    let options = Options::parse();
    Ok(options)
}

/// Runs the viewer program.
fn run_program() -> Result<()> {
    let options = parse_args()?;
    logging::initialize_logging(options.log_level.into());

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
