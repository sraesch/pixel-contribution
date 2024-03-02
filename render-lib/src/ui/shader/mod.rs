use nalgebra_glm::{Mat3, Vec4};

use crate::{Result, Shader, Uniform};

/// The UI shader.
pub struct UIShader {
    basic_shader: Shader,

    uniform_transform: Uniform,
    uniform_color: Uniform,
}

impl UIShader {
    /// Creates a new non-initialized UI shader.
    pub fn new() -> Self {
        let basic_shader = Shader::new();

        Self {
            basic_shader,
            uniform_transform: Default::default(),
            uniform_color: Default::default(),
        }
    }

    /// Initializes the UI shader.
    pub fn initialize(&mut self) -> Result<()> {
        let basic_vert_code = include_str!("basic.vert");
        let basic_frag_code = include_str!("basic.frag");

        self.basic_shader.load(basic_vert_code, basic_frag_code)?;
        self.uniform_transform = self.basic_shader.get_uniform("uniform_transform_mat")?;
        self.uniform_color = self.basic_shader.get_uniform("uniform_color")?;

        Ok(())
    }

    /// Binds the basic shader and sets the transformation matrix and color.
    ///
    /// # Arguments
    /// * `transform` - The transformation matrix.
    /// * `color` - The color.
    pub fn apply_basic_shader(&self, transform: &Mat3, color: &Vec4) {
        self.basic_shader.bind();
        self.uniform_transform.set_matrix3(transform);
        self.uniform_color.set_vector4(color);
    }
}
