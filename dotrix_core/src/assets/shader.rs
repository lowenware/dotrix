//! Shader Asset
use crate::renderer::{Renderer, ShaderModule};

/// Shader Asset
#[derive(Default)]
pub struct Shader {
    /// Shader name
    pub name: String,
    /// Shader code
    pub code: String,
    /// Shader Module
    pub module: ShaderModule,
}

impl Shader {
    /// Loads the shader to GPU
    pub fn load(&mut self, renderer: &Renderer) {
        if !self.module.loaded() {
            renderer.load_shader_module(&mut self.module, &self.name, &self.code);
        }
    }

    /// Returns true if shader is loaded
    pub fn loaded(&self) -> bool {
        self.module.loaded()
    }
}
