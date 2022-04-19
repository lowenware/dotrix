//! Shader Asset
use crate::renderer::{Renderer, ShaderModule};

/// Shader Asset
#[derive(Default)]
pub struct Shader {
    /// Shader name
    pub name: String, // TODO: set module.label instead
    /// Shader code
    pub code: String,
    /// Shader Module
    pub module: ShaderModule,
}

impl Shader {
    /// Loads the shader to GPU
    pub fn load(&mut self, renderer: &Renderer) {
        if !self.module.loaded() {
            self.module.label = self.name.clone();
            renderer.load_shader(&mut self.module, &self.code);
        }
    }

    /// Returns true if shader is loaded
    pub fn loaded(&self) -> bool {
        self.module.loaded()
    }
}
