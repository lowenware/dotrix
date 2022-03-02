use super::Context;
use std::borrow::Cow;
use wgpu;

/// Shader Module
pub struct ShaderModule {
    /// Shader Label
    pub label: String,
    /// WGPU Shader Module
    pub wgpu_shader_module: Option<wgpu::ShaderModule>,
}

impl ShaderModule {
    /// Constructs new shader module
    pub fn new(label: &str) -> Self {
        Self {
            label: String::from(label),
            wgpu_shader_module: None,
        }
    }

    /// Load shader module to GPU
    pub fn load(&mut self, ctx: &Context, source: &str) {
        self.wgpu_shader_module = Some(ctx.device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: Some(&self.label),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
            },
        ));
    }

    /// Returns true if shader module was loaded to GPU
    pub fn loaded(&self) -> bool {
        self.wgpu_shader_module.is_some()
    }

    /// Unloads the sahder module from GPU
    pub fn unload(&mut self) {
        self.wgpu_shader_module.take();
    }

    /// Get unwrapped reference to WGPU Shader Module
    pub fn get(&self) -> &wgpu::ShaderModule {
        self.wgpu_shader_module
            .as_ref()
            .expect("Shader module must be loaded")
    }
}

impl Default for ShaderModule {
    fn default() -> Self {
        Self {
            label: String::from("Noname shader module"),
            wgpu_shader_module: None,
        }
    }
}
