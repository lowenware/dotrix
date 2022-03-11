use crate::{Grid, TexSdf};
use dotrix_core::{
    assets::Shader,
    ecs::{Const, Mut},
    renderer::{
        wgpu, Access, BindGroup, Binding, Buffer, Compute, ComputeArgs, ComputeOptions,
        PipelineLayout, Stage, Texture as TextureBuffer, WorkGroups,
    },
    Assets, Renderer, World,
};
use std::convert::TryInto;

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
    pub ping_buffer: TextureBuffer,
    pub pong_buffer: TextureBuffer,
    pub init_pipeline: Option<Compute>,
    pub jumpflood_pipelines: Vec<Compute>,
    pub jumpflood_data: Vec<Buffer>,
    pub sdf_pipeline: Option<Compute>,
    pub debug_buffer: Option<Buffer>,
}

impl Default for VoxelJumpFlood {
    fn default() -> Self {
        Self {
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
            jumpflood_data: vec![],
            sdf_pipeline: None,
            debug_buffer: Some(Buffer::map_read("Debug buffer")),
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
    }
}

/// Uniform structure for ancillary data of the jump flood calculation
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
struct Data {
    k: u32,
    padding: [f32; 3],
}
unsafe impl bytemuck::Zeroable for Data {}
unsafe impl bytemuck::Pod for Data {}

pub(super) fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let mut shader = Shader {
        name: String::from(JUMP_FLOOD_PIPELINE),
        code: String::from(include_str!("./jump_flood/jump_flood.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, JUMP_FLOOD_PIPELINE);

    let mut shader = Shader {
        name: String::from(VOXEL_TO_JUMP_FLOOD_PIPELINE),
        code: String::from(include_str!("./jump_flood/jump_flood_voxel_seed.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, VOXEL_TO_JUMP_FLOOD_PIPELINE);

    let mut shader = Shader {
        name: String::from(JUMP_FLOOD_TO_DF_PIPELINE),
        code: String::from(include_str!("./jump_flood/jump_flood_df.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, JUMP_FLOOD_TO_DF_PIPELINE);
}

async fn print_debug_buffer(buffer: Buffer) {
    let wgpu_buffer = buffer.wgpu_buffer.expect("Buffer must be loaded");
    let buffer_slice = wgpu_buffer.slice(..);
    // Gets the future representing when `staging_buffer` can be read from
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

    if let Ok(()) = buffer_future.await {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to f32
        let result: Vec<f32> = data
            .chunks_exact(4)
            .map(|b| f32::from_ne_bytes(b.try_into().unwrap()))
            .collect();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        wgpu_buffer.unmap(); // Unmaps buffer from memory
                             // If you are familiar with C++ these 2 lines can be thought of similarly to:
                             //   delete myPointer;
                             //   myPointer = NULL;
                             // It effectively frees the memory
                             //
        println!("Data: {:?}", result);
    }
}

// Compute the SDF from the grid
pub(super) fn compute(world: Const<World>, assets: Const<Assets>, mut renderer: Mut<Renderer>) {
    for (grid, jump_flood, sdf) in world.query::<(&mut Grid, &mut VoxelJumpFlood, &mut TexSdf)>() {
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

                println!("Compute Seed");
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

                let bytes_per_pixel = 4 * 4;

                renderer.create_buffer(
                    jump_flood.debug_buffer.as_mut().unwrap(),
                    grid.dimensions[0] * grid.dimensions[1] * grid.dimensions[2] * bytes_per_pixel,
                );
                renderer.copy_texture_to_buffer(
                    &jump_flood.ping_buffer,
                    jump_flood.debug_buffer.as_ref().unwrap(),
                    grid.dimensions,
                    bytes_per_pixel,
                );

                jump_flood.init_pipeline = Some(voxel_to_jump_flood);
            }
        } else if let Some(debug_buffer) = jump_flood.debug_buffer.take() {
            std::thread::spawn(move || {
                futures::executor::block_on(print_debug_buffer(debug_buffer))
            });
        }

        if jump_flood.init_pipeline.is_some() && jump_flood.jumpflood_pipelines.is_empty() {
            let n = *grid.dimensions.iter().max().unwrap();

            let mut ping_buffer = &jump_flood.ping_buffer;
            let mut pong_buffer = &jump_flood.pong_buffer;

            for i in 0..((n as f32).log2().ceil() as usize) {
                let k = n / 2u32.pow(i as u32 + 1);

                let mut buffer = Buffer::uniform("Jump Flood Params");
                let data = Data {
                    k,
                    padding: Default::default(),
                };
                renderer.load_buffer(&mut buffer, bytemuck::cast_slice(&[data]));

                let mut jump_flood_compute: Compute = Default::default();

                if jump_flood_compute.pipeline.shader.is_null() {
                    jump_flood_compute.pipeline.shader = assets
                        .find::<Shader>(JUMP_FLOOD_PIPELINE)
                        .unwrap_or_default();
                }

                if let Some(shader) = assets.get(jump_flood_compute.pipeline.shader) {
                    if !shader.loaded() {
                        continue;
                    }

                    renderer.bind(
                        &mut jump_flood_compute.pipeline,
                        PipelineLayout::Compute {
                            label: "JumpFlood".into(),
                            shader,
                            bindings: &[BindGroup::new(
                                "Globals",
                                vec![
                                    Binding::Uniform("Params", Stage::Compute, &buffer),
                                    Binding::Texture3D("VoxelTexture", Stage::Compute, ping_buffer),
                                    Binding::StorageTexture3D(
                                        "InitSeeds",
                                        Stage::Compute,
                                        pong_buffer,
                                        Access::WriteOnly,
                                    ),
                                ],
                            )],
                            options: ComputeOptions { cs_main: "main" },
                        },
                    );

                    println!("Compute JumpFlood:{}", k);
                    renderer.compute(
                        &mut jump_flood_compute.pipeline,
                        &ComputeArgs {
                            work_groups: WorkGroups {
                                x: workgroup_size_x,
                                y: workgroup_size_y,
                                z: workgroup_size_z,
                            },
                        },
                    );

                    jump_flood.jumpflood_pipelines.push(jump_flood_compute);
                    jump_flood.jumpflood_data.push(buffer);
                    (ping_buffer, pong_buffer) = (pong_buffer, ping_buffer);
                }
            }

            // SDF conversion
            if jump_flood.sdf_pipeline.is_none() {
                sdf.load(&renderer, grid);

                let mut jump_flood_df: Compute = Default::default();

                if jump_flood_df.pipeline.shader.is_null() {
                    jump_flood_df.pipeline.shader = assets
                        .find::<Shader>(JUMP_FLOOD_TO_DF_PIPELINE)
                        .unwrap_or_default();
                }

                if let Some(shader) = assets.get(jump_flood_df.pipeline.shader) {
                    if !shader.loaded() {
                        continue;
                    }

                    renderer.bind(
                        &mut jump_flood_df.pipeline,
                        PipelineLayout::Compute {
                            label: "JumpFlood_2_SDF".into(),
                            shader,
                            bindings: &[BindGroup::new(
                                "Globals",
                                vec![
                                    Binding::Texture3D("Voxel", Stage::Compute, &grid.buffer),
                                    Binding::Texture3D("JumpFlood", Stage::Compute, pong_buffer),
                                    Binding::StorageTexture3D(
                                        "SDF",
                                        Stage::Compute,
                                        &sdf.buffer,
                                        Access::WriteOnly,
                                    ),
                                ],
                            )],
                            options: ComputeOptions { cs_main: "main" },
                        },
                    );

                    println!("Compute DF");
                    renderer.compute(
                        &mut jump_flood_df.pipeline,
                        &ComputeArgs {
                            work_groups: WorkGroups {
                                x: workgroup_size_x,
                                y: workgroup_size_y,
                                z: workgroup_size_z,
                            },
                        },
                    );

                    jump_flood.sdf_pipeline = Some(jump_flood_df);
                }
            }
        }
    }
}
