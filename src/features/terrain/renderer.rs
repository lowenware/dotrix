use std::collections::HashMap;
use std::io::Cursor;

use ash::vk::Handle;

use crate::graphics::vk;
use crate::graphics::{Buffer, CommandRecorder, Display, Extent2D, Gpu, RenderSubmit};
use crate::models::{Transform3D, VertexBufferLayout, VertexNormal, VertexPosition};
use crate::{Any, Camera, Entity, Frame, Id, Ref, Task, World};

use super::{
    LoD, Moisture, SpawnTerrainOutput, Terrain, DEFAULT_TILES_IN_VIEW_RANGE, DEFAULT_TILE_SIZE,
};

pub type MeshLayout = (VertexPosition, VertexNormal, Moisture);

/// Terrain rendering task
pub struct RenderTerrain {
    /// Wait for render submits from following
    wait_for: Vec<std::any::TypeId>,
    /// GPU instance
    gpu: Gpu,
    /// Version of surface to track changes and update framebuffers and fender pass
    surface_version: u64,
    /// globals buffer
    globals_buffer: Buffer,
    /// transform buffer
    instance_buffer: Buffer,
    /// indirect buffer
    indirect_buffer: Buffer,
    /// Indices buffer
    index_buffer: Buffer,
    /// Vertex buffer
    vertex_buffer: Buffer,
    /// Used bytes in vertex buffer
    vertex_buffer_usage: u64,
    /// descriptor sets
    descriptor_sets: Vec<vk::DescriptorSet>,
    /// descriptor pool
    _descriptor_pool: vk::DescriptorPool,
    /// descriptor set layouts
    _desc_set_layouts: [vk::DescriptorSetLayout; 1],
    /// Pipeline layout to render models
    pipeline_layout_render: vk::PipelineLayout,
    /// Graphics pipeline to render models
    pipeline_render: vk::Pipeline,
    /// Vertex shader module
    shader_vertex: vk::ShaderModule,
    /// Fragment shader module
    shader_fragment: vk::ShaderModule,
    /// Terrain tiles index
    tiles_index: HashMap<Id<Entity>, Slot>,
    /// Lod info index
    lods_index: HashMap<LoD, LodInfo>,
    /// Instance buffer data
    instance_buffer_data: Vec<InstanceUniform>,
    /// Indirect buffer data
    indirect_buffer_data: Vec<vk::DrawIndexedIndirectCommand>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Slot {
    /// Terrain tile offset in the buffer
    offset: u64,
    /// Size
    size: u64,
}

#[derive(Clone, Debug, Default)]
pub struct LodSetup {
    pub lod: LoD,
    pub vertices_count: u32,
    pub indices: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Default)]
struct LodInfo {
    /// Number of vertices
    vertices_count: u32,
    /// Number of indices
    indices_count: u32,
    /// Base index of the LoD in buffer
    first_index: u32,
    /// Number of tiles
    tiles_count: u32,
}

impl Drop for RenderTerrain {
    fn drop(&mut self) {
        unsafe {
            self.gpu.device_wait_idle().unwrap();

            // pipelines
            if self.pipeline_render != vk::Pipeline::null() {
                self.gpu.destroy_pipeline(self.pipeline_render);
            }

            // pipelines layouts
            self.gpu
                .destroy_pipeline_layout(self.pipeline_layout_render);

            // shaders
            self.gpu.destroy_shader_module(self.shader_vertex);
            self.gpu.destroy_shader_module(self.shader_fragment);

            // buffers
            self.globals_buffer.free_memory_and_destroy(&self.gpu);
            self.index_buffer.free_memory_and_destroy(&self.gpu);
            self.vertex_buffer.free_memory_and_destroy(&self.gpu);
            self.indirect_buffer.free_memory_and_destroy(&self.gpu);
            self.instance_buffer.free_memory_and_destroy(&self.gpu);

            // descriptors
            for &descriptor_set_layout in self._desc_set_layouts.iter() {
                self.gpu
                    .destroy_descriptor_set_layout(descriptor_set_layout);
            }
            self.gpu.destroy_descriptor_pool(self._descriptor_pool);
        };
    }
}

impl Task for RenderTerrain {
    type Output = RenderSubmit;
    type Context = (
        Any<SpawnTerrainOutput>,
        Any<Camera>,
        Any<Frame>,
        Ref<Display>,
        Ref<World>,
    );

    fn run(
        &mut self,
        (spawn_terrain_output, camera, frame, display, world): Self::Context,
    ) -> Self::Output {
        let globals_uniform = [self.globals_uniform(&camera)];
        unsafe {
            self.globals_buffer
                .map_and_write_to_device_memory(&self.gpu, 0, &globals_uniform);
        }
        // verify pipeline
        if let Some(surface_version) = display.surface_changed(self.surface_version) {
            unsafe {
                log::debug!("resize: Surface changed");
                self.gpu.device_wait_idle().unwrap();

                // rebuild pipelines
                if self.pipeline_render.is_null() {
                    log::debug!("resize: create_graphics_pipelines");
                    self.pipeline_render = self.create_graphics_pipelines(
                        display.render_pass(),
                        display.surface_resolution(),
                    )[0];
                }
            };
            self.surface_version = surface_version;
        }

        // clear for new cycle
        self.instance_buffer_data.clear();
        self.indirect_buffer_data.clear();

        // TODO: to calculate UVs for terrain color samplier we will need to have global offset for
        // X and Z to subtract from tile coordinates

        // 1. Mark slots occupied by exiled terrain as free
        let mut free_slots = spawn_terrain_output
            .tiles_to_exile
            .iter()
            .map(|i| {
                self.tiles_index
                    .remove(i)
                    .expect("Terrain index is corrupted")
            })
            .collect::<Vec<_>>();

        // 2. Iterate over terrain in the world and store in buffer missing meshes
        let vertex_size = MeshLayout::vertex_size();

        let mut lods_counters = HashMap::<LoD, u32>::new();

        for (id, terrain) in world.query::<(&Id<Entity>, &Terrain)>() {
            if !spawn_terrain_output.scene.contains(id) {
                log::error!("ECS_TERRAIN: garbage entity: {:?}", id);
                continue;
            }
            let lod_info = match self.lods_index.get(&terrain.lod) {
                Some(lod_info) => lod_info,
                None => panic!("No LoD info for {:?}", terrain.lod),
            };
            let instance = InstanceUniform {
                transform: Transform3D::default().matrix().to_cols_array_2d(),
            };
            self.instance_buffer_data.push(instance);

            if !self.tiles_index.contains_key(id) {
                let mesh_buffer_size = (vertex_size as u64) * (lod_info.vertices_count as u64);
                let slot = self.find_free_slot(&mut free_slots, mesh_buffer_size);
                let reuse_slot = slot.is_some();
                let slot = slot.unwrap_or(Slot {
                    offset: self.vertex_buffer_usage,
                    size: mesh_buffer_size,
                });

                if !reuse_slot && self.vertex_buffer_usage >= self.vertex_buffer.size {
                    log::error!(
                        "tiles: Vertex buffer will overflow by {} bytes of mesh (lod={})!",
                        mesh_buffer_size,
                        terrain.lod.value()
                    );
                }
                self.tiles_index.insert(*id, slot);
                // write to vertex buffer
                let mesh_data = terrain
                    .mesh
                    .buffer::<MeshLayout>()
                    .expect("Terrain mesh must have required attributes");

                let counter = lods_counters.entry(terrain.lod).or_insert(0);

                *counter += 1;
                let vertex_buffer_usage = unsafe {
                    self.vertex_buffer.map_and_write_to_device_memory(
                        &self.gpu,
                        slot.offset,
                        mesh_data.as_slice(),
                    )
                };

                if !reuse_slot {
                    self.vertex_buffer_usage += vertex_buffer_usage;
                }
            }

            let slot = match self.tiles_index.get(id) {
                Some(slot) => slot,
                None => panic!("Terrain slot information was not found for {id:?}"),
            };

            let instance_number = self.indirect_buffer_data.len() as u32;
            self.indirect_buffer_data
                .push(vk::DrawIndexedIndirectCommand {
                    instance_count: 1,
                    first_instance: instance_number,
                    first_index: lod_info.first_index,
                    index_count: lod_info.indices_count,
                    vertex_offset: (slot.offset / (vertex_size as u64)) as i32,
                });
        }

        unsafe {
            self.instance_buffer.map_and_write_to_device_memory(
                &self.gpu,
                0,
                self.instance_buffer_data.as_slice(),
            )
        };

        unsafe {
            self.indirect_buffer.map_and_write_to_device_memory(
                &self.gpu,
                0,
                self.indirect_buffer_data.as_slice(),
            );
        };
        // 3. Prepare and execute render pass

        let command_recorder = Recorder {
            resolution: frame.resolution,
            draw_count: self.indirect_buffer_data.len() as u32,
            vertex_buffer: self.vertex_buffer.handle,
            index_buffer: self.index_buffer.handle,
            indirect_buffer: self.indirect_buffer.handle,
            descriptor_sets: self.descriptor_sets.clone(),
            pipeline_layout: self.pipeline_layout_render,
            pipeline: self.pipeline_render,
        };

        RenderSubmit::new::<Self>(Box::new(command_recorder), &self.wait_for)
    }
}

impl RenderTerrain {
    pub fn setup() -> RenderTerrainSetup {
        RenderTerrainSetup::default()
    }

    pub fn new(display: &mut Display, setup: RenderTerrainSetup) -> Self {
        let terrain_tiles_capacity = 4 * setup.tiles_in_view_range * setup.tiles_in_view_range;

        let gpu = display.gpu();

        let globals_buffer = unsafe {
            Self::create_globals_uniform_buffer(&gpu)
                .expect("Could not allocate globals uniform buffer")
        };

        let instance_buffer = unsafe {
            Self::create_storage_buffer(
                &gpu,
                (terrain_tiles_capacity as u64) * (std::mem::size_of::<InstanceUniform>() as u64),
            )
            .expect("Could not allocate instance storage buffer")
        };

        let indirect_buffer = unsafe {
            Self::create_indirect_buffer(
                &gpu,
                (terrain_tiles_capacity as u64)
                    * (std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64),
            )
            .expect("Could not allocate indirect buffer")
        };

        let mut index_buffer_data = Vec::new();
        let lods_index = Self::generate_lods_index(
            setup.tiles_in_view_range,
            setup.lods,
            &mut index_buffer_data,
        );

        let vertex_buffer_size = lods_index
            .values()
            .map(|lod_info| {
                (lod_info.vertices_count as u64)
                    * (lod_info.tiles_count as u64)
                    * (MeshLayout::vertex_size() as u64)
            })
            .sum();

        let index_buffer = unsafe {
            Self::create_index_buffer(
                &gpu,
                (index_buffer_data.len() * std::mem::size_of::<u32>()) as u64,
            )
            .expect("Could not allocate index buffer")
        };
        // write indices to index buffer
        unsafe {
            index_buffer.map_and_write_to_device_memory(&gpu, 0, index_buffer_data.as_slice());
        };

        let vertex_buffer = unsafe {
            Self::create_vertex_buffer(&gpu, vertex_buffer_size)
                .expect("Could not allocate vertex buffer (non-rigged)")
        };

        // bindings layout
        let descriptor_sizes = [
            // Globals
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
            // Instances
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            },
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&descriptor_sizes)
            .max_sets(1);
        let descriptor_pool = unsafe { gpu.create_descriptor_pool(&descriptor_pool_info).unwrap() };

        let desc_layout_bindings = [
            // Globals
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            // Instances
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];
        let descriptor_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&desc_layout_bindings);

        let desc_set_layouts =
            unsafe { [gpu.create_descriptor_set_layout(&descriptor_info).unwrap()] };

        let desc_alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_set_layouts);
        let descriptor_sets = unsafe { gpu.allocate_descriptor_sets(&desc_alloc_info).unwrap() };

        let globals_uniform_buffer_descriptor = vk::DescriptorBufferInfo {
            buffer: globals_buffer.handle,
            offset: 0,
            range: globals_buffer.size,
        };

        let instance_storage_buffer_descriptor = vk::DescriptorBufferInfo {
            buffer: instance_buffer.handle,
            offset: 0,
            range: instance_buffer.size,
        };

        let write_desc_sets = [
            vk::WriteDescriptorSet {
                dst_binding: 0,
                dst_set: descriptor_sets[0],
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_buffer_info: &globals_uniform_buffer_descriptor,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_binding: 1,
                dst_set: descriptor_sets[0],
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                p_buffer_info: &instance_storage_buffer_descriptor,
                ..Default::default()
            },
        ];

        unsafe {
            gpu.update_descriptor_sets(&write_desc_sets, &[]);
        };

        // pipeline layout
        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&desc_set_layouts);
        let pipeline_layout_render = unsafe {
            gpu.create_pipeline_layout(&pipeline_layout_create_info)
                .expect("Failed to create non-rigged pipeline layout")
        };

        let shader_vertex = unsafe {
            Self::load_shader_module(&gpu, include_bytes!("shaders/terrain.vert.spv"))
                .expect("Failed to load terrain vertex shader module")
        };
        let shader_fragment = unsafe {
            Self::load_shader_module(&gpu, include_bytes!("shaders/terrain.frag.spv"))
                .expect("Failed to load terrain fragment shader module")
        };

        Self {
            wait_for: vec![],
            gpu,
            surface_version: 0,
            globals_buffer,
            instance_buffer,
            indirect_buffer,
            index_buffer,
            vertex_buffer,
            vertex_buffer_usage: 0,

            descriptor_sets,
            _descriptor_pool: descriptor_pool,
            _desc_set_layouts: desc_set_layouts,
            pipeline_layout_render,
            pipeline_render: vk::Pipeline::null(),
            shader_vertex,
            shader_fragment,

            tiles_index: HashMap::with_capacity(16),
            lods_index,

            instance_buffer_data: Vec::with_capacity(terrain_tiles_capacity as usize),
            indirect_buffer_data: Vec::with_capacity(terrain_tiles_capacity as usize),
        }
    }

    fn find_free_slot(&self, slots: &mut Vec<Slot>, size: u64) -> Option<Slot> {
        if let Some(slot_index) =
            slots
                .iter()
                .enumerate()
                .find_map(|(index, slot)| if slot.size == size { Some(index) } else { None })
        {
            let slot = slots.remove(slot_index);
            return Some(slot);
        }
        None
    }

    fn generate_lods_index(
        tiles_in_view_range: u32,
        lods: Vec<LodSetup>,
        index_buffer_data: &mut Vec<u32>,
    ) -> HashMap<LoD, LodInfo> {
        let mut result = HashMap::with_capacity(lods.len());
        let mut index_buffer_offset = 0;
        let mut tiles_per_side_in_lod = 2 * tiles_in_view_range;
        for lod_setup in lods.into_iter() {
            let max_tiles_of_lod = tiles_per_side_in_lod * tiles_per_side_in_lod;
            let tiles_count = if lod_setup.lod.value() != 0 {
                let tiles_per_side_in_higher_lod = tiles_per_side_in_lod - 2;
                tiles_per_side_in_lod -= 2;
                max_tiles_of_lod - tiles_per_side_in_higher_lod * tiles_per_side_in_higher_lod
            } else {
                max_tiles_of_lod
            };

            let lod_info = LodInfo {
                vertices_count: lod_setup.vertices_count,
                indices_count: lod_setup.indices.len() as u32,
                first_index: (index_buffer_offset / (std::mem::size_of::<u32>() as u64)) as u32,
                tiles_count,
            };

            index_buffer_data.extend(lod_setup.indices.into_iter());
            index_buffer_offset =
                (index_buffer_data.len() as u64) * (std::mem::size_of::<u32>() as u64);
            result.insert(lod_setup.lod, lod_info);
        }

        result
    }

    /// Returns Buffer, binded memory and allocated size
    unsafe fn create_globals_uniform_buffer(gpu: &Gpu) -> Result<Buffer, vk::Result> {
        let buffer_create_info = vk::BufferCreateInfo {
            size: std::mem::size_of::<GlobalsUniform>() as u64,
            usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        Buffer::create_and_allocate(gpu, &buffer_create_info)
    }

    unsafe fn create_vertex_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
        let buffer_create_info = vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        Buffer::create_and_allocate(gpu, &buffer_create_info)
    }

    unsafe fn create_index_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
        let buffer_create_info = vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::INDEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        Buffer::create_and_allocate(gpu, &buffer_create_info)
    }

    unsafe fn create_storage_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
        let buffer_create_info = vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::STORAGE_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        Buffer::create_and_allocate(gpu, &buffer_create_info)
    }

    unsafe fn create_indirect_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
        let buffer_create_info = vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::INDIRECT_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        Buffer::create_and_allocate(gpu, &buffer_create_info)
    }

    unsafe fn load_shader_module(gpu: &Gpu, bytes: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
        // let bytes = include_bytes!("shaders/non-rigged.frag.spv");
        let mut cursor = Cursor::new(bytes);
        let shader_code = ash::util::read_spv(&mut cursor).expect("Failed to read shader SPV code");
        let shader_module_create_info = vk::ShaderModuleCreateInfo::default().code(&shader_code);

        gpu.create_shader_module(&shader_module_create_info)
    }

    unsafe fn create_graphics_pipelines(
        &self,
        render_pass: vk::RenderPass,
        surface_resolution: Extent2D,
    ) -> Vec<vk::Pipeline> {
        let shader_entry_point = c"main";

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                module: self.shader_vertex,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: self.shader_fragment,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<MeshLayout>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let vertex_input_attribute_descriptions = [
            // position
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            // normal
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::size_of::<VertexPosition>() as u32,
            },
            // moisture
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                // format: vk::Format::R32G32B32A32_SFLOAT,
                format: vk::Format::R32_SFLOAT,
                offset: std::mem::size_of::<VertexPosition>() as u32
                    + std::mem::size_of::<VertexNormal>() as u32,
            },
        ];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions);
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: surface_resolution.width as f32,
            height: surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [(vk::Extent2D {
            width: surface_resolution.width,
            height: surface_resolution.height,
        })
        .into()];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            // polygon_mode: vk::PolygonMode::LINE,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::FRONT,
            ..Default::default()
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let noop_stencil_state = vk::StencilOpState {
            // fail_op: vk::StencilOp::KEEP,
            // pass_op: vk::StencilOp::KEEP,
            // depth_fail_op: vk::StencilOp::KEEP,
            // compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(self.pipeline_layout_render)
            .render_pass(render_pass);

        self.gpu
            .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info])
            .expect("Failed to create graphics pipelines")
    }

    pub fn globals_uniform(&self, camera: &Camera) -> GlobalsUniform {
        // let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 800.0 / 600.0, 1.0, 10.0);
        // let view = Mat4::look_at_rh(Vec3::new(1.5f32, -5.0, 3.0), Vec3::ZERO, Vec3::Z);
        GlobalsUniform {
            proj: camera.proj.to_cols_array_2d(),
            view: camera.view.to_cols_array_2d(),
        }
    }
}

pub struct RenderTerrainSetup {
    tile_size: u32,
    tiles_in_view_range: u32,
    lods: Vec<LodSetup>,
}

impl Default for RenderTerrainSetup {
    fn default() -> Self {
        Self {
            tile_size: DEFAULT_TILE_SIZE,
            tiles_in_view_range: DEFAULT_TILES_IN_VIEW_RANGE,
            lods: vec![],
        }
    }
}

impl RenderTerrainSetup {
    pub fn tile_size(mut self, value: u32) -> Self {
        self.tile_size = value;
        self
    }

    pub fn tiles_in_view_range(mut self, value: u32) -> Self {
        self.tiles_in_view_range = value;
        self
    }

    pub fn lods(mut self, lods: Vec<LodSetup>) -> Self {
        self.lods = lods;
        self
    }

    pub fn create(self, display: &mut Display) -> RenderTerrain {
        RenderTerrain::new(display, self)
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct GlobalsUniform {
    /// Projection matrix
    pub proj: [[f32; 4]; 4],
    /// View matrix
    pub view: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct InstanceUniform {
    /// Terrain instance local transform matrix
    pub transform: [[f32; 4]; 4],
}

pub struct Recorder {
    resolution: Extent2D,
    draw_count: u32,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    indirect_buffer: vk::Buffer,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_sets: Vec<vk::DescriptorSet>,
}

impl CommandRecorder for Recorder {
    unsafe fn record(&self, gpu: &Gpu, command_buffer: vk::CommandBuffer) {
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.resolution.width as f32,
            height: self.resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [(vk::Extent2D {
            width: self.resolution.width,
            height: self.resolution.height,
        })
        .into()];

        gpu.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            &self.descriptor_sets[..],
            &[],
        );

        // ONLY MESH
        if self.draw_count != 0 {
            gpu.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
            gpu.cmd_set_viewport(command_buffer, 0, &viewports);
            gpu.cmd_set_scissor(command_buffer, 0, &scissors);
            gpu.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer], &[0]);

            gpu.cmd_bind_index_buffer(command_buffer, self.index_buffer, 0, vk::IndexType::UINT32);

            gpu.cmd_draw_indexed_indirect(
                command_buffer,
                self.indirect_buffer,
                0,
                self.draw_count,
                std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u32,
            );
        }
    }
}
