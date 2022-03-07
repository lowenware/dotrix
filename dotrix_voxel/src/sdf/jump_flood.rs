use crate::Grid;
use dotrix_core::{
    assets::Shader,
    ecs::{Const, Mut},
    renderer::{wgpu, Texture as TextureBuffer},
    Assets, Globals, Renderer, World,
};

/// Component for generating a SDF
/// which tells the renderer how far
/// a point is from the surface.
/// Computed with the jump flooding
/// algorithm, which is an approximate
/// algorithm with O(log(n)) complexity
pub struct JumpFlood {
    /// 3D Texture buffer
    pub buffer: TextureBuffer,
}

const PIPELINE_LABEL: &str = "dotrix_voxel::sdf::jump_flood";

fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: String::from(include_str!("./jump_flood.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, PIPELINE_LABEL);
}

// Compute the SDF from the grid
fn compute(world: Const<World>, assets: Const<Assets>, mut renderer: Mut<Renderer>) {
    for (grid, jump_flood) in world.query::<(&Grid, &mut JumpFlood)>() {}
}
