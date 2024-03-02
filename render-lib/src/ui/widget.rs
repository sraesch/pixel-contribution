use nalgebra_glm::{scaling2d, Mat3, Vec2, Vec3, Vec4};

use super::Shape;

/// A widget is a UI element that can be rendered.
pub struct Widget {
    pos: Vec2,
    size: Vec2,
    color: Vec4,
    shape: Shape,
}

impl Widget {
    /// Creates a new widget with the given position and size.
    ///
    /// # Arguments
    /// * `pos` - The position of the widget.
    /// * `size` - The size of the widget.
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        let default_color = Vec4::new(1f32, 1f32, 1f32, 0.5f32);
        let rectangle = Shape::rectangle();

        Self {
            pos,
            size,
            color: default_color,
            shape: rectangle,
        }
    }

    /// Creates a new widget with the given position and size.
    ///
    /// # Arguments
    /// * `pos` - The position of the widget.
    /// * `size` - The size of the widget.
    /// * `shape` - The shape of the widget.
    pub fn new_with_shape(pos: Vec2, size: Vec2, shape: Shape) -> Self {
        let default_color = Vec4::new(1f32, 1f32, 1f32, 0.5f32);

        Self {
            pos,
            size,
            color: default_color,
            shape,
        }
    }

    /// Creates and returns the transformation matrix for the widget.
    pub fn create_transform_matrix(&self) -> Mat3 {
        let mut transform = scaling2d(&self.size);

        transform.set_column(2, &Vec3::new(self.pos.x, self.pos.y, 1f32));

        transform
    }

    /// Sets the color of the widget.
    ///
    /// # Arguments
    /// * `color` - The new color of the widget.
    #[inline]
    pub fn set_color(&mut self, color: Vec4) {
        self.color = color;
    }

    /// Sets the position of the widget.
    ///
    /// # Arguments
    /// * `pos` - The new position of the widget.
    #[inline]
    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    /// Sets the size of the widget.
    ///
    /// # Arguments
    /// * `size` - The new size of the widget.
    #[inline]
    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    /// Returns the position of the widget.
    #[inline]
    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    /// Returns the size of the widget.
    #[inline]
    pub fn size(&self) -> Vec2 {
        self.size
    }

    /// Returns the color of the widget.
    #[inline]
    pub fn color(&self) -> Vec4 {
        self.color
    }

    /// Renders the widget.
    pub fn render(&self) {
        self.shape.render();
    }

    /// Returns a reference onto the shape of the widget.
    #[inline]
    pub fn get_shape(&self) -> &Shape {
        &self.shape
    }

    /// Returns a reference onto the shape of the widget.
    #[inline]
    pub fn get_shape_mut(&mut self) -> &mut Shape {
        &mut self.shape
    }
}
