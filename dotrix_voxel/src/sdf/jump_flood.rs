/// Use the jump flood algorithm to convert
/// a voxel into a distance field
///
/// Paper:
/// Jump Flooding in GPU with Applications to Voronoi Diagram and Distance Transform
/// Guodong Rong et al.
///
/// In 2006 ACM Symposium on Interactive 3D
/// Graphics and Games, 14-17 March, Redwood City,
/// CA, USA, pp. 109-116, pp. 228.
///
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

const VOXEL_TO_JUMP_FLOOD_PIPELINE: &str = "dotrix_voxel::sdf::jump_flood_voxel_seed";
const JUMP_FLOOD_PIPELINE: &str = "dotrix_voxel::sdf::jump_flood";
const JUMP_FLOOD_TO_DF_PIPELINE: &str = "dotrix_voxel::sdf::jump_flood_sdf";
const VOXELS_PER_WORKGROUP: [usize; 3] = [8, 8, 4];
const SCALE_FACTOR: u32 = 4;

/// Jump flood varients
/// detailed in section 3.3.1 of the original paper
pub enum JumpFlood {
    /// Standard (fastest, most errors)
    Jfa,
    /// 1 Additional round
    Jfa1,
    /// 2 Additional rounds
    Jfa2,
    /// log2(n) additional rounds (slowest, least errors)
    JfaSquare,
}

/// Component for generating a SDF
/// which tells the renderer how far
/// a point is from the surface.
/// Computed with the jump flooding
/// algorithm, which is an approximate
/// algorithm with O(log2(n)) complexity
pub struct VoxelJumpFlood {
    jump_flood_varient: JumpFlood,
    ping_buffer: TextureBuffer,
    pong_buffer: TextureBuffer,
    init_pipeline: Option<Compute>,
    jumpflood_pipelines: Vec<Compute>,
    jumpflood_data: Vec<Buffer>,
    sdf_pipeline: Option<Compute>,
    last_revision: Option<u32>,
}

impl Default for VoxelJumpFlood {
    fn default() -> Self {
        Self::new()
    }
}

impl VoxelJumpFlood {
    /// Use default parameters for a new jump flood
    pub fn new() -> Self {
        Self {
            jump_flood_varient: JumpFlood::JfaSquare,
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
            last_revision: None,
        }
    }

    #[must_use]
    pub fn with_algorithm(mut self, jfa: JumpFlood) -> Self {
        self.jump_flood_varient = jfa;
        self
    }

    /// Reset the algorithm for recomputation, call this is the size of the grid has changed
    /// This does not need to be called if only the values of the grid have changes
    pub fn reset(&mut self) {
        self.init_pipeline = None;
        self.jumpflood_pipelines = vec![];
        self.jumpflood_data = vec![];
        self.sdf_pipeline = None;
        self.last_revision = None;
        self.ping_buffer = {
            let mut buffer = TextureBuffer::new_3d("PingBuffer")
                .use_as_storage()
                .allow_write();
            buffer.format = wgpu::TextureFormat::Rgba32Float;
            buffer
        };
        self.pong_buffer = {
            let mut buffer = TextureBuffer::new_3d("PongBuffer")
                .use_as_storage()
                .allow_write();
            buffer.format = wgpu::TextureFormat::Rgba32Float;
            buffer
        };
    }

    /// Load the voxel data for computation
    pub fn load(&mut self, renderer: &Renderer, grid: &Grid) {
        let pixel_size = 4 * 4;
        let dimensions = grid.get_dimensions();
        let dim: [u32; 3] = [
            dimensions[0] * SCALE_FACTOR,
            dimensions[1] * SCALE_FACTOR,
            dimensions[2] * SCALE_FACTOR,
        ];
        let data: Vec<Vec<u8>> =
            vec![0u8; pixel_size * dim[0] as usize * dim[1] as usize * dim[2] as usize]
                .chunks(dim[0] as usize * dim[1] as usize * pixel_size)
                .map(|chunk| chunk.to_vec())
                .collect();

        let slices: Vec<&[u8]> = data.iter().map(|chunk| chunk.as_slice()).collect();

        renderer.update_or_load_texture(&mut self.ping_buffer, dim[0], dim[1], slices.as_slice());

        renderer.update_or_load_texture(&mut self.pong_buffer, dim[0], dim[1], slices.as_slice());
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

// Compute the SDF from the grid
pub(super) fn compute(world: Const<World>, assets: Const<Assets>, mut renderer: Mut<Renderer>) {
    for (grid, jump_flood, sdf) in world.query::<(&mut Grid, &mut VoxelJumpFlood, &mut TexSdf)>() {
        let grid_dim = grid.get_dimensions();
        let dimensions: [u32; 3] = [
            grid_dim[0] * SCALE_FACTOR,
            grid_dim[1] * SCALE_FACTOR,
            grid_dim[2] * SCALE_FACTOR,
        ];
        let workgroup_size_x =
            (dimensions[0] as f32 / VOXELS_PER_WORKGROUP[0] as f32).ceil() as u32;
        let workgroup_size_y =
            (dimensions[1] as f32 / VOXELS_PER_WORKGROUP[1] as f32).ceil() as u32;
        let workgroup_size_z =
            (dimensions[2] as f32 / VOXELS_PER_WORKGROUP[2] as f32).ceil() as u32;

        // Set up pipelines once
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
                                Binding::Texture3D(
                                    "VoxelTexture",
                                    Stage::Compute,
                                    grid.get_buffer(),
                                ),
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

                jump_flood.init_pipeline = Some(voxel_to_jump_flood);
            }
        }

        if jump_flood.init_pipeline.is_some() && jump_flood.jumpflood_pipelines.is_empty() {
            let n = *dimensions.iter().max().unwrap();

            let mut ping_buffer = &jump_flood.ping_buffer;
            let mut pong_buffer = &jump_flood.pong_buffer;

            let n_log2 = (n as f32).log2().ceil() as u32;
            let n_ceil = 2u32.pow(n_log2);

            let limit = match jump_flood.jump_flood_varient {
                JumpFlood::Jfa => n_log2 as usize,
                JumpFlood::Jfa1 => n_log2 as usize + 1,
                JumpFlood::Jfa2 => n_log2 as usize + 2,
                JumpFlood::JfaSquare => n_log2 as usize * 2,
            };

            for i in 0..limit {
                let k = if i < n_log2 as usize {
                    n_ceil / 2u32.pow(i as u32 + 1)
                } else {
                    2u32.pow((i as u32) - n_log2)
                };

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
                                    Binding::Texture3D("Voxel", Stage::Compute, grid.get_buffer()),
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

                    jump_flood.sdf_pipeline = Some(jump_flood_sdf);
                }
            }
        }

        // Call compute when ever the grid changes
        if jump_flood.last_revision.is_none()
            || jump_flood.last_revision.unwrap() != grid.get_revision()
        {
            grid.load(&renderer, &assets);
            jump_flood.load(&renderer, grid);
            if let (Some(voxel_to_jump_flood), Some(jump_flood_sdf)) = (
                jump_flood.init_pipeline.as_mut(),
                jump_flood.sdf_pipeline.as_mut(),
            ) {
                jump_flood.last_revision = Some(grid.get_revision());

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

                for jump_flood_compute in jump_flood.jumpflood_pipelines.iter_mut() {
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
                }

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
            }
        }
    }
}
