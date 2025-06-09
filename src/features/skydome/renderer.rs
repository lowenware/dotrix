use std::io::Cursor;

use ash::vk::Handle;

use crate::graphics::vk;
use crate::graphics::{Buffer, CommandRecorder, Gpu, RenderSubmit};
use crate::math::Vec3;
use crate::models::{Transform3D, VertexNormal, VertexPosition};
use crate::{Any, Camera, Display, Entity, Extent2D, Frame, Id, Ref, Task, World};

use super::SkyDome;

pub type MeshLayout = (VertexPosition, VertexNormal);

pub struct RenderSkyDome {
    /// Current SkyDome entity
    entity_id: Id<Entity>,
    /// Wait for render submits from following
    wait_for: Vec<std::any::TypeId>,
    /// GPU instance
    gpu: Gpu,
    /// Version of surface to track changes and update framebuffers and fender pass
    surface_version: u64,
    /// globals buffer
    globals_buffer: Buffer,
    /// Indices buffer
    index_buffer: Buffer,
    /// Vertex buffer
    vertex_buffer: Buffer,
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
}

impl RenderSkyDome {
    pub fn setup() -> RenderSkyDomeSetup {
        RenderSkyDomeSetup::default()
    }

    pub fn new(display: &mut Display, setup: RenderSkyDomeSetup) -> Self {
        let gpu = display.gpu();

        let globals_buffer = unsafe {
            Self::create_globals_uniform_buffer(&gpu)
                .expect("Could not allocate globals uniform buffer")
        };

        let index_buffer = unsafe {
            Self::create_index_buffer(&gpu, setup.index_buffer_size)
                .expect("Could not allocate index buffer")
        };

        let vertex_buffer = unsafe {
            Self::create_vertex_buffer(&gpu, setup.vertex_buffer_size)
                .expect("Could not allocate vertex buffer (non-rigged)")
        };

        // bindings layout
        let descriptor_sizes = [
            // Globals
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
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

        let write_desc_sets = [vk::WriteDescriptorSet {
            dst_binding: 0,
            dst_set: descriptor_sets[0],
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            p_buffer_info: &globals_uniform_buffer_descriptor,
            ..Default::default()
        }];

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
            Self::load_shader_module(&gpu, include_bytes!("shaders/skydome.vert.spv"))
                .expect("Failed to load skydome vertex shader module")
        };
        let shader_fragment = unsafe {
            Self::load_shader_module(&gpu, include_bytes!("shaders/skydome.frag.spv"))
                .expect("Failed to load skydome fragment shader module")
        };

        Self {
            entity_id: Id::null(),
            wait_for: vec![],
            gpu,
            surface_version: 0,
            globals_buffer,
            index_buffer,
            vertex_buffer,

            descriptor_sets,
            _descriptor_pool: descriptor_pool,
            _desc_set_layouts: desc_set_layouts,
            pipeline_layout_render,
            pipeline_render: vk::Pipeline::null(),
            shader_vertex,
            shader_fragment,
        }
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
            cull_mode: vk::CullModeFlags::BACK,
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

    pub fn globals_uniform(&self, skydome: &SkyDome, camera: &Camera) -> GlobalsUniform {
        // let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 800.0 / 600.0, 1.0, 10.0);
        // let view = Mat4::look_at_rh(Vec3::new(1.5f32, -5.0, 3.0), Vec3::ZERO, Vec3::Z);
        let view = camera.view.to_cols_array_2d();
        log::debug!("view mx: {view:?}");

        let transform = Transform3D::new(
            Vec3::new(
                camera.target.x,
                skydome.transform.translate.y,
                camera.target.z,
            ),
            skydome.transform.rotate,
            skydome.transform.scale,
        );
        //
        GlobalsUniform {
            proj: camera.proj.to_cols_array_2d(),
            view,
            transform: transform.matrix().to_cols_array_2d(),
            horizon_color: (&skydome.horizon_color).into(),
            zenith_color: (&skydome.zenith_color).into(),
            extras: [skydome.size, skydome.transform.translate.y, 0.0, 0.0],
            _padding: [0.0; 4],
        }
    }

    unsafe fn load_shader_module(gpu: &Gpu, bytes: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
        // let bytes = include_bytes!("shaders/non-rigged.frag.spv");
        let mut cursor = Cursor::new(bytes);
        let shader_code = ash::util::read_spv(&mut cursor).expect("Failed to read shader SPV code");
        let shader_module_create_info = vk::ShaderModuleCreateInfo::default().code(&shader_code);

        gpu.create_shader_module(&shader_module_create_info)
    }
}

impl Drop for RenderSkyDome {
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

            // descriptors
            for &descriptor_set_layout in self._desc_set_layouts.iter() {
                self.gpu
                    .destroy_descriptor_set_layout(descriptor_set_layout);
            }
            self.gpu.destroy_descriptor_pool(self._descriptor_pool);
        }
    }
}

impl Task for RenderSkyDome {
    type Output = RenderSubmit;
    type Context = (Any<Camera>, Any<Frame>, Ref<Display>, Ref<World>);

    fn run(&mut self, (camera, frame, display, world): Self::Context) -> Self::Output {
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

        if let Some((id, skydome)) = world.query::<(&Id<Entity>, &SkyDome)>().next() {
            let indices = skydome
                .mesh
                .indices::<u32>()
                .expect("Sky dome mesh MUST be indexed");

            let index_count = indices.len() as u32;

            if *id != self.entity_id {
                let mesh_data = skydome
                    .mesh
                    .buffer::<MeshLayout>()
                    .expect("Terrain mesh must have required attributes");

                log::debug!(
                    "Required vertex buffer size: {}",
                    mesh_data.as_slice().len()
                );
                log::debug!("Required index buffer size: {}", indices.len() * 4);

                unsafe {
                    self.vertex_buffer.map_and_write_to_device_memory(
                        &self.gpu,
                        0,
                        mesh_data.as_slice(),
                    );
                    self.index_buffer
                        .map_and_write_to_device_memory(&self.gpu, 0, indices);
                }
            }

            let globals_uniform = [Self::globals_uniform(self, skydome, &camera)];

            unsafe {
                self.globals_buffer
                    .map_and_write_to_device_memory(&self.gpu, 0, &globals_uniform);
            }

            let command_recorder = Recorder {
                resolution: frame.resolution,
                vertex_buffer: self.vertex_buffer.handle,
                index_buffer: self.index_buffer.handle,
                pipeline_layout: self.pipeline_layout_render,
                pipeline: self.pipeline_render,
                descriptor_sets: self.descriptor_sets.clone(),
                index_count,
            };
            log::debug!("submit rendering");
            return RenderSubmit::new::<Self>(Box::new(command_recorder), &self.wait_for);
        }

        log::debug!("submit dummy rendering");
        RenderSubmit::skip::<Self>(self.wait_for.as_slice())
    }
}

pub struct Recorder {
    resolution: Extent2D,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_sets: Vec<vk::DescriptorSet>,
    index_count: u32,
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
        gpu.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline,
        );
        gpu.cmd_set_viewport(command_buffer, 0, &viewports);
        gpu.cmd_set_scissor(command_buffer, 0, &scissors);
        gpu.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer], &[0]);

        gpu.cmd_bind_index_buffer(command_buffer, self.index_buffer, 0, vk::IndexType::UINT32);

        let instance_count = 1;
        let first_index = 0;
        let vertex_offset = 0;
        let first_instance = 0;
        log::debug!("gpu.cmd_draw_indexed({})", self.index_count);

        gpu.cmd_draw_indexed(
            command_buffer,
            self.index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        );
    }
}

pub struct RenderSkyDomeSetup {
    pub vertex_buffer_size: u64,
    pub index_buffer_size: u64,
}

impl RenderSkyDomeSetup {
    pub fn vertex_buffer_size(mut self, value: u64) -> Self {
        self.vertex_buffer_size = value;
        self
    }

    pub fn index_buffer_size(mut self, value: u64) -> Self {
        self.index_buffer_size = value;
        self
    }

    pub fn create(self, display: &mut Display) -> RenderSkyDome {
        RenderSkyDome::new(display, self)
    }
}

impl Default for RenderSkyDomeSetup {
    fn default() -> Self {
        Self {
            vertex_buffer_size: 128 * 1024,
            index_buffer_size: 64 * 1024,
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct GlobalsUniform {
    /// Projection matrix
    pub proj: [[f32; 4]; 4],
    /// View matrix
    pub view: [[f32; 4]; 4],
    /// Transform matrix
    pub transform: [[f32; 4]; 4],
    /// horizon_color
    pub horizon_color: [f32; 4],
    /// zenith color
    pub zenith_color: [f32; 4],
    /// Size, padding
    pub extras: [f32; 4],
    /// Padding
    _padding: [f32; 4],
}
