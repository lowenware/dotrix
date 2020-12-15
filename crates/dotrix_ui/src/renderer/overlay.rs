use wgpu::util::DeviceExt;

use dotrix::{
    ecs::{Const, Mut, Context},
    components::SkyBox,
    renderer::Renderer,
    services::{Assets, Camera, World},
};

pub fn overlay_renderer(
    mut ctx: Context<SystemContext>,
    camera: Const<Camera>,
    assets: Const<Assets>,
    renderer: Mut<Renderer>,
    world: Const<World>
) {
    println!("Overlay renderer works!");
}

pub struct RendererContext {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub proj_view: wgpu::Buffer,
    pub sampler: wgpu::Sampler,
}

#[derive(Default)]
pub struct SystemContext {
    renderer_context: Option<RendererContext>,
}