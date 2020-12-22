pub use crate::{
    assets::{Id, Texture},
    services::Assets,
    renderer::skybox::RendererContext,
};

pub struct Buffers {
    pub bind_group: wgpu::BindGroup,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub indices_count: u32,
}

#[derive(Default)]
pub struct SkyBox {
    pub primary_texture: [Id<Texture>; 6],
    pub secondary_texture: Option<[Id<Texture>; 6]>,
    pub buffers: Option<Buffers>,
}

impl SkyBox {

}
