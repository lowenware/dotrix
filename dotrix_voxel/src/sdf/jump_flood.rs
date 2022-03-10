use crate::Grid;
use dotrix_core::{
    assets::Shader,
    ecs::{Const, Mut},
    renderer::{
        wgpu, Access, BindGroup, Binding, Buffer, Compute, ComputeArgs, ComputeOptions,
        PipelineLayout, Stage, Texture as TextureBuffer, WorkGroups,
    },
    Assets, Renderer, World,
};

const VOXEL_TO_JUMP_FLOOD_PIPELINE: &str = "dotrix_voxel::sdf::jump_flood_voxel_seed";
const JUMP_FLOOD_PIPELINE: &str = "dotrix_voxel::sdf::jump_flood";
const JUMP_FLOOD_TO_DF_PIPELINE: &str = "dotrix_voxel::sdf::jump_flood_df";
const VOXELS_PER_WORKGROUP: [usize; 3] = [8, 8, 4];

/// Component for generating a SDF
/// which tells the renderer how far
/// a point is from the surface.
/// Computed with the jump flooding
/// algorithm, which is an approximate
/// algorithm with O(log(n)) complexity
pub struct VoxelJumpFlood {
    pub data: Buffer,
    pub ping_buffer: TextureBuffer,
    pub pong_buffer: TextureBuffer,
    pub init_pipeline: Option<Compute>,
    pub jumpflood_pipelines: Vec<Compute>,
}

impl Default for VoxelJumpFlood {
    fn default() -> Self {
        Self {
            data: Buffer::uniform("Voxel-Jump Flood Params"),
            ping_buffer: {
                let mut buffer = TextureBuffer::new_3d("PingBuffer")
                    .use_as_storage()
                    .allow_write();
                buffer.format = wgpu::TextureFormat::Rgba32Float;
                buffer
            },
            pong_buffer: {
                let mut buffer = TextureBuffer::new_3d("PongBuffer")
                    .use_as_storage()
                    .allow_write();
                buffer.format = wgpu::TextureFormat::Rgba32Float;
                buffer
            },
            init_pipeline: None,
            jumpflood_pipelines: vec![],
        }
    }
}

impl VoxelJumpFlood {
    pub fn load(&mut self, renderer: &Renderer, grid: &Grid) {
        let pixel_size = 4 * 4;
        let data: Vec<Vec<u8>> = vec![
            0u8;
            pixel_size
                * grid.dimensions[0] as usize
                * grid.dimensions[1] as usize
                * grid.dimensions[2] as usize
        ]
        .chunks(grid.dimensions[0] as usize * grid.dimensions[1] as usize * pixel_size)
        .map(|chunk| chunk.to_vec())
        .collect();

        let slices: Vec<&[u8]> = data.iter().map(|chunk| chunk.as_slice()).collect();

        renderer.load_texture(
            &mut self.ping_buffer,
            grid.dimensions[0],
            grid.dimensions[1],
            slices.as_slice(),
        );

        renderer.load_texture(
            &mut self.pong_buffer,
            grid.dimensions[0],
            grid.dimensions[1],
            slices.as_slice(),
        );

        let data = Data {
            origin: grid.position,
            dimensions: grid.voxel_dimensions,
            padding: Default::default(),
        };
        renderer.load_buffer(&mut self.data, bytemuck::cast_slice(&[data]));
    }
}

/// Uniform structure for ancillary data of the jump flood calculation
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
struct Data {
    origin: [f32; 3],
    dimensions: [f32; 3],
    padding: [f32; 2],
}
unsafe impl bytemuck::Zeroable for Data {}
unsafe impl bytemuck::Pod for Data {}

pub(super) fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let mut shader = Shader {
        name: String::from(JUMP_FLOOD_PIPELINE),
        code: String::from(include_str!("./jump_flood.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, JUMP_FLOOD_PIPELINE);

    let mut shader = Shader {
        name: String::from(VOXEL_TO_JUMP_FLOOD_PIPELINE),
        code: String::from(include_str!("./jump_flood_voxel_seed.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, VOXEL_TO_JUMP_FLOOD_PIPELINE);
}

// Compute the SDF from the grid
pub(super) fn compute(world: Const<World>, assets: Const<Assets>, mut renderer: Mut<Renderer>) {
    for (grid, jump_flood) in world.query::<(&mut Grid, &mut VoxelJumpFlood)>() {
        let workgroup_size_x =
            (grid.dimensions[0] as f32 / VOXELS_PER_WORKGROUP[0] as f32).ceil() as u32;
        let workgroup_size_y =
            (grid.dimensions[1] as f32 / VOXELS_PER_WORKGROUP[1] as f32).ceil() as u32;
        let workgroup_size_z =
            (grid.dimensions[2] as f32 / VOXELS_PER_WORKGROUP[2] as f32).ceil() as u32;

        if jump_flood.init_pipeline.is_none() {
            grid.load(&renderer, &assets);
            jump_flood.load(&renderer, grid);

            let mut voxel_to_jump_flood: Compute = Default::default();

            if voxel_to_jump_flood.pipeline.shader.is_null() {
                voxel_to_jump_flood.pipeline.shader = assets
                    .find::<Shader>(VOXEL_TO_JUMP_FLOOD_PIPELINE)
                    .unwrap_or_default();
            }

            if let Some(shader) = assets.get(voxel_to_jump_flood.pipeline.shader) {
                if !shader.loaded() {
                    continue;
                }

                renderer.bind(
                    &mut voxel_to_jump_flood.pipeline,
                    PipelineLayout::Compute {
                        label: "Voxel_2_JumpFlood".into(),
                        shader,
                        bindings: &[BindGroup::new(
                            "Globals",
                            vec![
                                Binding::Uniform("Params", Stage::Compute, &jump_flood.data),
                                Binding::Texture3D("VoxelTexture", Stage::Compute, &grid.buffer),
                                Binding::StorageTexture3D(
                                    "InitSeeds",
                                    Stage::Compute,
                                    &jump_flood.ping_buffer,
                                    Access::WriteOnly,
                                ),
                            ],
                        )],
                        options: ComputeOptions { cs_main: "main" },
                    },
                );

                renderer.compute(
                    &mut voxel_to_jump_flood.pipeline,
                    &ComputeArgs {
                        work_groups: WorkGroups {
                            x: workgroup_size_x,
                            y: workgroup_size_y,
                            z: workgroup_size_z,
                        },
                    },
                );

                jump_flood.init_pipeline = Some(voxel_to_jump_flood);
            }
        }
    }
}
