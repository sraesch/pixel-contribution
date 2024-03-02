mod shader;

use crate::Result;

pub struct UI {
    shader: shader::UIShader,
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

        Self { shader }
    }

    /// Initializes the UI.
    pub fn initialize(&mut self) -> Result<()> {
        self.shader.initialize()?;

        Ok(())
    }
}
