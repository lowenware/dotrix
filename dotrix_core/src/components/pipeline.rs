use crate::{
    assets::Shader,
    generics::Id,
    renderer::{ Bindings, Renderer, UniformBuffer },
};

use dotrix_math::{ Vec3, Quat, Mat4, SquareMatrix };

/// Rendering control component
#[derive(Default)]
pub struct Pipeline {
    /// [`Id`] of the shader
    pub shader: Id<Shader>,
    /// Rendering bindings
    pub bindings: Bindings,
    /// rendering cycle
    pub cycle: usize,
    /// is rendering disabled
    pub disabled: bool,
}

impl Pipeline {
    pub fn cycle(&self, renderer: &Renderer) -> bool {
        let result = !self.disabled && self.cycle != renderer.cycle() && !self.shader.is_null();
        result
    }
}
