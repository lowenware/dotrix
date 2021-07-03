use std::collections::HashMap;

use crate::{
    ecs::{ Const, Mut },
    generics::Id,
    assets::{ Assets, Shader },
    renderer::{ Renderer, Bindings, VertexBuffer, UniformBuffer, TextureBuffer, Sampler, PipelineBackend },
};


#[derive(Default)]
pub struct Pipelines {
    map: HashMap<Id<Pipeline>, PipelineEntry>,
    loaded: bool,
    last_id: u64,
}

impl Pipelines {
    pub fn new() -> Self {
        Self::default()
    }

}


