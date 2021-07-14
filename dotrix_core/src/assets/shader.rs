use crate::renderer::{ Renderer, ShaderModule };

#[derive(Default)]
pub struct Shader {
    pub name: String,
    pub code: String,
    pub module: ShaderModule,
}

impl Shader {
    pub fn load(&mut self, renderer: &Renderer) {
        if !self.module.is_loaded() {
            renderer.load_shader_module(&mut self.module, &self.name, &self.code);
        }
    }
}
