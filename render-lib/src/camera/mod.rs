mod camera_data;

pub use camera_data::*;
use math::BoundingSphere;
use nalgebra_glm::{mat4_to_mat3, rotation, Vec3};
use winit::event::MouseButton;

use crate::Result;

/// The current camera operation mode.
#[derive(Debug)]
enum Mode {
    Nothing,
    Zoom,
    Move,
    Rotate,
}

/// A camera object that is moving around a defined center with a specific radius.
pub struct Camera {
    /// The internal camera data
    data: CameraData,

    /// The current camera operation.
    mode: Mode,

    /// The last mouse cursor position.
    save_cursor: [f64; 2],

    /// The saved camera data before the current operation.
    saved_data: CameraData,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            data: CameraData::new(),
            mode: Mode::Nothing,
            save_cursor: [0.0, 0.0],
            saved_data: CameraData::new(),
        }
    }

    /// Updates the window size
    ///
    ///* `w` - The new width of the window
    ///* `h` - The new height of the window
    pub fn update_window_size(&mut self, w: u32, h: u32) {
        self.data.set_window_size(w, h);
    }

    /// Mouse button event that starts or stops a camera operation.
    ///
    /// # Arguments
    /// `x` - The x coordinate of the mouse cursor when the operation starts or stops.
    /// `y` - The y coordinate of the mouse cursor when the operation starts or stops.
    /// `btn` - The mouse button that was pressed or released.
    /// `pressed` - True if the mouse button was pressed, false if it was released.
    pub fn event_mouse_button(&mut self, x: f64, y: f64, btn: MouseButton, pressed: bool) {
        if pressed {
            self.save_cursor[0] = x;
            self.save_cursor[1] = y;

            self.saved_data = self.data;

            match btn {
                MouseButton::Right => self.mode = Mode::Zoom,
                MouseButton::Middle => self.mode = Mode::Move,
                MouseButton::Left => self.mode = Mode::Rotate,
                _ => {}
            }
        } else {
            self.modify(x, y);
            self.mode = Mode::Nothing;
        }
    }

    /// Mouse motion event that the current camera operation modifies.
    ///
    /// # Arguments
    /// `x` - The x coordinate of the mouse cursor.
    /// `y` - The y coordinate of the mouse cursor.
    pub fn event_mouse_motion(&mut self, x: f64, y: f64) {
        self.modify(x, y);
    }

    /// Sets the internal camera radius around the center.
    ///
    ///* `radius` - The new radius.
    #[inline]
    pub fn set_radius(&mut self, radius: f32) {
        self.data.set_radius(radius.ln())
    }

    /// Focuses the camera on the given bounding sphere.
    ///
    ///* `bounding_sphere` - The bounding sphere to focus on.
    pub fn focus(&mut self, bounding_sphere: &BoundingSphere) -> Result<()> {
        let center = bounding_sphere.center;

        self.set_radius(bounding_sphere.radius * 1.5);

        let camera_data = &mut self.data;
        camera_data.set_center(&center);

        camera_data.set_scene(bounding_sphere.center, bounding_sphere.radius)?;

        Ok(())
    }

    /// Modifies the camera data according to the current operation.
    ///
    /// # Arguments
    /// `new_x` - The x coordinate of the mouse cursor.
    /// `new_y` - The y coordinate of the mouse cursor.
    fn modify(&mut self, new_x: f64, new_y: f64) {
        let x_drift_func =
            || ((new_x - self.save_cursor[0]) as f32) / (self.data.get_window_size().0 as f32);

        let y_drift_func =
            || ((new_y - self.save_cursor[1]) as f32) / (self.data.get_window_size().1 as f32);

        match self.mode {
            Mode::Zoom => {
                let ydiff = y_drift_func() * 2.0;
                let new_radius = self.saved_data.get_radius() + ydiff;
                self.data.set_radius(new_radius);
            }
            Mode::Move => {
                let cam_axis = self.data.get_axis();

                let x_axis: Vec3 = cam_axis.column(0).into();
                let y_axis: Vec3 = cam_axis.column(1).into();

                let factor = self.data.get_radius().exp();

                let x_drift = -x_drift_func() * factor;
                let y_drift = y_drift_func() * factor;

                let new_center =
                    *self.saved_data.get_center() + x_axis * x_drift + y_axis * y_drift;
                self.data.set_center(&new_center);
            }
            Mode::Rotate => {
                let x_drift = x_drift_func();
                let y_drift = y_drift_func();

                let cam_axis = self.saved_data.get_axis();

                let xrot_mat = rotation(-x_drift * 2.5, &cam_axis.column(1).into());
                let yrot_mat = rotation(-y_drift * 2.5, &cam_axis.column(0).into());

                let rot_mat = yrot_mat * xrot_mat;

                self.data
                    .set_rotated_cam_axis(self.saved_data.get_axis(), &mat4_to_mat3(&rot_mat));
            }
            Mode::Nothing => {}
        }
    }

    /// Returns reference onto the internal camera data
    pub fn get_data(&self) -> &CameraData {
        &self.data
    }

    /// Returns mutable reference onto the internal camera data
    pub fn get_data_mut(&mut self) -> &mut CameraData {
        &mut self.data
    }
}
