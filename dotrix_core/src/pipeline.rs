use crate::{
    assets::Shader,
    id::Id,
    renderer::{Bindings, Options, Renderer, ScissorsRect},
};

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
    /// Pipeline Options
    pub options: Options,
}

impl Pipeline {
    /// Checks if rendering cycle should be performed
    pub fn cycle(&self, renderer: &Renderer) -> bool {
        !self.disabled && self.cycle != renderer.cycle() && !self.shader.is_null()
    }

    /// Returns true if Pipeline is ready to run
    pub fn ready(&self) -> bool {
        self.bindings.loaded()
    }

    /// Adds scissors rectangle for rendering
    #[must_use]
    pub fn with_scissors_rect(
        mut self,
        clip_min_x: u32,
        clip_min_y: u32,
        width: u32,
        height: u32,
    ) -> Self {
        self.options.scissors_rect = Some(ScissorsRect {
            clip_min_x,
            clip_min_y,
            width,
            height,
        });
        self
    }
}
