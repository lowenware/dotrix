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
const JUMP_FLOOD_TO_DF_PIPELINE: &str = "dotrix_voxel::sdf::jump_flood_sdf";
const VOXELS_PER_WORKGROUP: [usize; 3] = [8, 8, 4];
const SCALE_FACTOR: u32 = 2;

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
            debug_buffer: None,
        }
    }
}

impl VoxelJumpFlood {
    pub fn load(&mut self, renderer: &Renderer, grid: &Grid) {
        let pixel_size = 4 * 4;
        let dim: [u32; 3] = [
            grid.dimensions[0] * SCALE_FACTOR,
            grid.dimensions[1] * SCALE_FACTOR,
            grid.dimensions[2] * SCALE_FACTOR,
        ];
        let data: Vec<Vec<u8>> =
            vec![0u8; pixel_size * dim[0] as usize * dim[1] as usize * dim[2] as usize]
                .chunks(dim[0] as usize * dim[1] as usize * pixel_size)
                .map(|chunk| chunk.to_vec())
                .collect();

        let slices: Vec<&[u8]> = data.iter().map(|chunk| chunk.as_slice()).collect();

        renderer.load_texture(&mut self.ping_buffer, dim[0], dim[1], slices.as_slice());

        renderer.load_texture(&mut self.pong_buffer, dim[0], dim[1], slices.as_slice());
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
        code: String::from(include_str!("./jump_flood/jump_flood_sdf.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, JUMP_FLOOD_TO_DF_PIPELINE);
}

async fn print_debug_buffer(buffer: Buffer, dimensions: [u32; 3], channels: u32) {
    let bytes_per_pixel: u32 = channels * 4;
    let unpadded_bytes_per_row: u32 =
        std::num::NonZeroU32::new(bytes_per_pixel as u32 * dimensions[0])
            .unwrap()
            .into();
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
    let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
    let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;

    let wgpu_buffer = buffer.wgpu_buffer.expect("Buffer must be loaded");
    let buffer_slice = wgpu_buffer.slice(..);
    // Gets the future representing when `staging_buffer` can be read from
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

    if let Ok(()) = buffer_future.await {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to f32
        let result: Vec<Vec<Vec<Vec<f32>>>> = data
            .chunks_exact((padded_bytes_per_row * dimensions[1]) as usize)
            .map(|img| {
                let rows: Vec<Vec<Vec<f32>>> = img
                    .chunks_exact(padded_bytes_per_row as usize)
                    .map(|row| {
                        let row_f32: Vec<f32> = row
                            .chunks_exact(4)
                            .map(|b| f32::from_ne_bytes(b.try_into().unwrap()))
                            .collect();
                        let pixels: Vec<Vec<f32>> = row_f32
                            .chunks(channels as usize)
                            .map(|pixels| pixels.to_vec())
                            .collect();
                        pixels[0..(dimensions[0] as usize)].to_vec()
                    })
                    .collect();
                rows
            })
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
        for (idx, img) in result.iter().enumerate() {
            println!("Z={}", idx);
            for row in img.iter() {
                println!("{:.2?}", row);
            }
        }
    }
}

#[allow(dead_code)]
enum DebugThing {
    None,
    Init,
    Jfa,
    Sdf,
}

// Compute the SDF from the grid
pub(super) fn compute(world: Const<World>, assets: Const<Assets>, mut renderer: Mut<Renderer>) {
    let debug_thing: DebugThing = DebugThing::None;
    for (grid, jump_flood, sdf) in world.query::<(&mut Grid, &mut VoxelJumpFlood, &mut TexSdf)>() {
        let dimensions: [u32; 3] = [
            grid.dimensions[0] * SCALE_FACTOR,
            grid.dimensions[1] * SCALE_FACTOR,
            grid.dimensions[2] * SCALE_FACTOR,
        ];
        let workgroup_size_x =
            (dimensions[0] as f32 / VOXELS_PER_WORKGROUP[0] as f32).ceil() as u32;
        let workgroup_size_y =
            (dimensions[1] as f32 / VOXELS_PER_WORKGROUP[1] as f32).ceil() as u32;
        let workgroup_size_z =
            (dimensions[2] as f32 / VOXELS_PER_WORKGROUP[2] as f32).ceil() as u32;

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

                jump_flood.init_pipeline = Some(voxel_to_jump_flood);

                if let DebugThing::Init = debug_thing {
                    let bytes_per_pixel = 4 * 4;
                    jump_flood.debug_buffer = Some(Buffer::map_read("Debug buffer"));
                    let unpadded_bytes_per_row: u32 =
                        std::num::NonZeroU32::new(bytes_per_pixel as u32 * dimensions[0])
                            .unwrap()
                            .into();
                    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
                    let padded_bytes_per_row_padding =
                        (align - unpadded_bytes_per_row % align) % align;
                    let padded_bytes_per_row =
                        unpadded_bytes_per_row + padded_bytes_per_row_padding;

                    renderer.create_buffer(
                        jump_flood.debug_buffer.as_mut().unwrap(),
                        padded_bytes_per_row * dimensions[1] * dimensions[2],
                    );
                    renderer.copy_texture_to_buffer(
                        &jump_flood.ping_buffer,
                        jump_flood.debug_buffer.as_ref().unwrap(),
                        dimensions,
                        bytes_per_pixel,
                    );
                }
            }
        } else if let DebugThing::Init = debug_thing {
            if let Some(debug_buffer) = jump_flood.debug_buffer.take() {
                let dim: [u32; 3] = dimensions;
                std::thread::spawn(move || {
                    futures::executor::block_on(print_debug_buffer(debug_buffer, dim, 4))
                });
            }
        }

        if jump_flood.init_pipeline.is_some() && jump_flood.jumpflood_pipelines.is_empty() {
            let n = *dimensions.iter().max().unwrap();

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
                sdf.load(&renderer, &dimensions);

                let mut jump_flood_sdf: Compute = Default::default();

                if jump_flood_sdf.pipeline.shader.is_null() {
                    jump_flood_sdf.pipeline.shader = assets
                        .find::<Shader>(JUMP_FLOOD_TO_DF_PIPELINE)
                        .unwrap_or_default();
                }

                if let Some(shader) = assets.get(jump_flood_sdf.pipeline.shader) {
                    if !shader.loaded() {
                        continue;
                    }

                    renderer.bind(
                        &mut jump_flood_sdf.pipeline,
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
                        &mut jump_flood_sdf.pipeline,
                        &ComputeArgs {
                            work_groups: WorkGroups {
                                x: workgroup_size_x,
                                y: workgroup_size_y,
                                z: workgroup_size_z,
                            },
                        },
                    );

                    jump_flood.sdf_pipeline = Some(jump_flood_sdf);

                    if let DebugThing::Jfa = debug_thing {
                        let bytes_per_pixel = 4 * 4;
                        jump_flood.debug_buffer = Some(Buffer::map_read("Debug buffer"));
                        let unpadded_bytes_per_row: u32 =
                            std::num::NonZeroU32::new(bytes_per_pixel as u32 * dimensions[0])
                                .unwrap()
                                .into();
                        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
                        let padded_bytes_per_row_padding =
                            (align - unpadded_bytes_per_row % align) % align;
                        let padded_bytes_per_row =
                            unpadded_bytes_per_row + padded_bytes_per_row_padding;

                        renderer.create_buffer(
                            jump_flood.debug_buffer.as_mut().unwrap(),
                            padded_bytes_per_row * dimensions[1] * dimensions[2],
                        );
                        renderer.copy_texture_to_buffer(
                            pong_buffer,
                            jump_flood.debug_buffer.as_ref().unwrap(),
                            dimensions,
                            bytes_per_pixel,
                        );
                    } else if let DebugThing::Sdf = debug_thing {
                        let bytes_per_pixel = 4 * 2;
                        jump_flood.debug_buffer = Some(Buffer::map_read("Debug buffer"));
                        let unpadded_bytes_per_row: u32 =
                            std::num::NonZeroU32::new(bytes_per_pixel as u32 * dimensions[0])
                                .unwrap()
                                .into();
                        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
                        let padded_bytes_per_row_padding =
                            (align - unpadded_bytes_per_row % align) % align;
                        let padded_bytes_per_row =
                            unpadded_bytes_per_row + padded_bytes_per_row_padding;

                        renderer.create_buffer(
                            jump_flood.debug_buffer.as_mut().unwrap(),
                            padded_bytes_per_row * dimensions[1] * dimensions[2],
                        );
                        renderer.copy_texture_to_buffer(
                            &sdf.buffer,
                            jump_flood.debug_buffer.as_ref().unwrap(),
                            dimensions,
                            bytes_per_pixel,
                        );
                    }
                }
            }
        } else if let DebugThing::Jfa = debug_thing {
            if let Some(debug_buffer) = jump_flood.debug_buffer.take() {
                let dim: [u32; 3] = dimensions;
                std::thread::spawn(move || {
                    futures::executor::block_on(print_debug_buffer(debug_buffer, dim, 4))
                });
            }
        } else if let DebugThing::Sdf = debug_thing {
            if let Some(debug_buffer) = jump_flood.debug_buffer.take() {
                let dim: [u32; 3] = dimensions;
                std::thread::spawn(move || {
                    futures::executor::block_on(print_debug_buffer(debug_buffer, dim, 2))
                });
            }
        }
    }
}
