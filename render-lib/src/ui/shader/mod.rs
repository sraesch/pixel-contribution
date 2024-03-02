use crate::{Result, Shader};

/// The UI shader.
pub struct UIShader {
    basic_shader: Shader,
}

impl UIShader {
    /// Creates a new non-initialized UI shader.
    pub fn new() -> Self {
        let basic_shader = Shader::new();

        Self { basic_shader }
    }

    /// Initializes the UI shader.
    pub fn initialize(&mut self) -> Result<()> {
        let basic_vert_code = include_str!("basic.vert");
        let basic_frag_code = include_str!("basic.frag");

        self.basic_shader.load(basic_vert_code, basic_frag_code)?;

        Ok(())
    }
}
