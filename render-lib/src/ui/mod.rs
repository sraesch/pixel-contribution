mod shader;
mod shape;
mod widget;

use std::collections::BTreeMap;

use nalgebra_glm::{scaling2d, Mat3, Vec2, Vec3};
pub use shape::*;
pub use widget::*;

use crate::{BlendFactor, FrameBuffer, Result};

pub struct UI {
    shader: shader::UIShader,
    width: f32,
    height: f32,

    widget_id_counter: u32,
    widgets: BTreeMap<u32, Widget>,
}

impl Default for UI {
    fn default() -> Self {
        Self::new()
    }
}

impl UI {
    /// Creates a new non-initialized UI.
    pub fn new() -> Self {
        let shader = shader::UIShader::new();

        Self {
            shader,
            width: 0.0,
            height: 0.0,
            widget_id_counter: 0,
            widgets: BTreeMap::new(),
        }
    }

    /// Initializes the UI.
    ///
    /// # Arguments
    /// * `width` - The width of the window containing the UI.
    /// * `height` - The height of the window containing the UI.
    pub fn initialize(&mut self, width: f32, height: f32) -> Result<()> {
        self.shader.initialize()?;

        self.width = width;
        self.height = height;

        Ok(())
    }

    /// Updates the size of the window containing the UI.
    ///
    /// # Arguments
    /// * `width` - The new width of the window.
    /// * `height` - The new height of the window.
    pub fn update_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Renders the UI.
    pub fn render(&self) -> Result<()> {
        FrameBuffer::depthtest(false);

        // activate alpha-transparency
        FrameBuffer::set_blending(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);

        // render all widgets
        let ui_transform = self.create_ui_transform_matrix();
        for widget in self.widgets.values() {
            let transform = ui_transform * widget.create_transform_matrix();
            let color = widget.color();

            self.shader.apply_basic_shader(&transform, &color);
            widget.render();
        }

        FrameBuffer::disable_blend();
        FrameBuffer::depthtest(true);

        Ok(())
    }

    /// Adds the given widget to the UI and returns its ID.
    /// Note: The widget will be drawn in the order of their addition.
    ///
    /// # Arguments
    /// * `widget` - The widget to add.
    pub fn add_widget(&mut self, widget: Widget) -> u32 {
        let id = self.widget_id_counter;
        self.widget_id_counter += 1;

        self.widgets.insert(id, widget);

        id
    }

    /// Removes the widget with the given ID from the UI.
    ///
    /// # Arguments
    /// * `id` - The ID of the widget to remove.
    #[inline]
    pub fn remove_widget(&mut self, id: u32) -> bool {
        self.widgets.remove(&id).is_some()
    }

    /// Creates and returns the transformation matrix for the UI.
    fn create_ui_transform_matrix(&self) -> Mat3 {
        let mut transform = scaling2d(&Vec2::new(2.0 / self.width, -2.0 / self.height));

        transform.set_column(2, &Vec3::new(-1.0, 1.0, 1f32));

        transform
    }
}
