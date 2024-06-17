use std::collections::HashMap;
use std::ffi::CStr;
use std::hash::Hash;
use std::io::Cursor;
use std::mem::size_of;

use ash::vk::DrawIndirectCommand;

use crate::graphics::vk;
use crate::graphics::{Buffer, RenderPass};
use crate::loaders::Assets;
use crate::log;
use crate::math::{Mat4, Vec3};
use crate::utils::{BufferLayout, Id, LayoutInBuffer, MeshLayout, MeshVerticesLayout};
use crate::world::{Entity, World};
use crate::{Any, Asset, Display, Extent2D, Frame, Gpu, Ref, Task};

use super::materials::MaterialUniform;
use super::{Armature, Material, Mesh, Transform, VertexNormal, VertexPosition, VertexTexture};

/* TODO:
 * [ ] Lights
 * [ ] Shadows
 * [ ] Textures
 * [ ] Move camera to the engine level
 * [ ] Indexed draw calls (add index buffer, separate indirect buffer)
 *     - Always use indexed drawing
 *     - If mesh has no indices, generate the index buffer as 0,1,2...VertexCount
 * [ ] Support for rigged meshes (second pipeline? second vertex buffer?)
 *
 **/

pub struct RenderModels {
    /// GPU instance
    gpu: Gpu,
    /// Wait for these semaphores before executing command buffers
    wait_semaphores: Vec<vk::Semaphore>,
    /// Signal these semaphores after rendering is done
    signal_semaphore: vk::Semaphore,
    /// Command Pool
    command_pool: vk::CommandPool,
    /// Setup command buffer
    command_buffer_setup: vk::CommandBuffer,
    /// Setup command buffer reuse fence
    command_buffer_setup_reuse_fence: vk::Fence,
    /// Draw command buffer
    command_buffer_draw: vk::CommandBuffer,
    /// Draw command buffer reuse fence
    command_buffer_draw_reuse_fence: vk::Fence,
    /// Framebuffers
    framebuffers: Vec<vk::Framebuffer>,
    /// Render pass
    render_pass: vk::RenderPass,
    /// Version of surface to track changes and update framebuffers and fender pass
    surface_version: u64,
    /// Index of instances by mesh (non rigged)
    instances_non_rigged: HashMap<Id<Mesh>, Vec<InstanceUniform>>,
    /// Index of instances by mesh (rigged)
    instances_rigged: HashMap<Id<Mesh>, Vec<InstanceUniform>>,
    /// Globals uniform buffer
    globals_buffer: Buffer,
    /// Indices buffer (rigged and non-rigged)
    index_buffer: Buffer,
    /// Used bytes in index buffer,
    index_buffer_used_size: u64,
    /// Vertex buffer (non-rigged)
    vertex_buffer_non_rigged: Buffer,
    /// Vertex buffer (rigged)
    vertex_buffer_rigged: Buffer,
    /// Indirect buffer
    /// Indirect buffer (non-rigged)
    indirect_buffer_non_rigged: Buffer,
    /// Indirect buffer (rigged)
    indirect_buffer_rigged: Buffer,
    /// Instances buffer (non-rigged)
    instances_buffer_non_rigged: Buffer,
    /// Instances buffer data (non-rigged)
    instances_buffer_non_rigged_data: Vec<InstanceUniform>,
    /// Instances buffer (rigged)
    instances_buffer_rigged: Buffer,
    /// Instances buffer data (rigged)
    instances_buffer_rigged_data: Vec<InstanceUniform>,
    /// Materials buffer
    materials_buffer: Buffer,
    /// Mapping of material index in the buffer by its ID
    materials_buffer_index: HashMap<Id<Material>, u32>,
    /// Materials buffer data
    materials_buffer_data: Vec<MaterialUniform>,
    /// Mesh layouts of non-rigged models
    mesh_layout_non_rigged: BufferLayout<Id<Mesh>, MeshLayout>,
    /// descriptor sets
    descriptor_sets: Vec<vk::DescriptorSet>,
    /// descriptor pool
    descriptor_pool: vk::DescriptorPool,
    /// descriptor set layouts
    desc_set_layouts: [vk::DescriptorSetLayout; 1],
    /// Pipeline layout to render non-rigged models
    pipeline_layout_render_non_rigged: vk::PipelineLayout,
    /// Graphics pipeline to render non-rigged models
    pipeline_render_non_rigged: vk::Pipeline,
    /// Vertex shader module for non-rigged pipeline
    shader_vertex_non_rigged: vk::ShaderModule,
    /// Fragment shader module for non-rigged pipeline
    shader_fragment_non_rigged: vk::ShaderModule,
}

pub type VertexBufferLayoutNonRigged = (VertexPosition, VertexNormal, VertexTexture);

impl Drop for RenderModels {
    fn drop(&mut self) {
        unsafe {
            self.gpu.device_wait_idle().unwrap();

            // pipelines
            self.destroy_graphics_pipelines();

            // pipelines layouts
            self.gpu
                .destroy_pipeline_layout(self.pipeline_layout_render_non_rigged);

            // shaders
            self.gpu
                .destroy_shader_module(self.shader_vertex_non_rigged);
            self.gpu
                .destroy_shader_module(self.shader_fragment_non_rigged);

            // buffers
            self.globals_buffer.free_memory_and_destroy(&self.gpu);
            self.index_buffer.free_memory_and_destroy(&self.gpu);
            self.vertex_buffer_non_rigged
                .free_memory_and_destroy(&self.gpu);
            self.vertex_buffer_rigged.free_memory_and_destroy(&self.gpu);
            self.indirect_buffer_non_rigged
                .free_memory_and_destroy(&self.gpu);
            self.indirect_buffer_rigged
                .free_memory_and_destroy(&self.gpu);
            self.instances_buffer_non_rigged
                .free_memory_and_destroy(&self.gpu);
            self.instances_buffer_rigged
                .free_memory_and_destroy(&self.gpu);
            self.materials_buffer.free_memory_and_destroy(&self.gpu);

            // descriptors
            for &descriptor_set_layout in self.desc_set_layouts.iter() {
                self.gpu
                    .destroy_descriptor_set_layout(descriptor_set_layout);
            }
            self.gpu.destroy_descriptor_pool(self.descriptor_pool);

            // framebuffers
            self.destroy_framebuffers();

            // render pass
            self.gpu.destroy_render_pass(self.render_pass);

            // command buffers
            self.gpu.destroy_command_pool(self.command_pool);

            // fences
            self.gpu
                .destroy_fence(self.command_buffer_setup_reuse_fence);
            self.gpu.destroy_fence(self.command_buffer_draw_reuse_fence);

            // semaphores
            self.gpu.destroy_semaphore(self.signal_semaphore);
        }
    }
}

impl Task for RenderModels {
    type Context = (Any<Frame>, Ref<Assets>, Ref<Display>, Ref<World>);
    type Output = RenderPass;

    fn run(&mut self, (frame, assets, display, world): Self::Context) -> Self::Output {
        log::debug!("pbr: begin");

        if let Some(surface_version) = display.surface_changed(self.surface_version) {
            unsafe {
                log::debug!("resize: Surface changed");
                self.gpu.device_wait_idle().unwrap();

                // rebuild framebuffers

                log::debug!("resize: destroy_framebuffers");
                self.destroy_framebuffers();

                log::debug!("resize: create_framebuffers");
                self.create_framebuffers(&display, self.render_pass);

                // rebuild pipelines
                if self.pipeline_render_non_rigged == vk::Pipeline::null() {
                    log::debug!("resize: destroy_graphics_pipelines");
                    self.destroy_graphics_pipelines();
                    log::debug!("resize: create_graphics_pipelines");
                    self.pipeline_render_non_rigged =
                        self.create_graphics_pipelines(display.surface_resolution());

                    // NOTE: the setup buffer should be probably a part of the Display
                    log::debug!("resize: setup_depth_image");
                    self.setup_depth_image(&display);
                }

                log::debug!("resize: complete -> {}", surface_version);
            };
            self.surface_version = surface_version;
        }

        let draw_count = self.update_buffers(&assets, &world);

        unsafe {
            self.execute_render_pass(&frame, draw_count);
            self.submit_draw_commands();
        }

        log::debug!("pbr: submit_command_buffer");
        RenderPass {}
    }
}

impl RenderModels {
    pub fn setup() -> RenderModelsSetup {
        RenderModelsSetup::default()
    }

    pub fn new(display: &mut Display, setup: RenderModelsSetup) -> Self {
        let gpu = display.gpu();
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(gpu.queue_family_index());
        let command_pool = unsafe {
            gpu.create_command_pool(&pool_create_info)
                .expect("Failed to create a command pool")
        };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(2)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let (command_buffer_setup, command_buffer_draw) = unsafe {
            gpu.allocate_command_buffers(&command_buffer_allocate_info)
                .into()
        };

        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let command_buffer_setup_reuse_fence = unsafe { gpu.create_fence(&fence_create_info) };
        let command_buffer_draw_reuse_fence = unsafe { gpu.create_fence(&fence_create_info) };

        let signal_semaphore_create_info = vk::SemaphoreCreateInfo::default();
        let signal_semaphore = unsafe {
            gpu.create_semaphore(&signal_semaphore_create_info)
                .expect("Failed to create a signal semaphore")
        };
        let mut wait_semaphores = setup.wait_semaphores;
        unsafe {
            wait_semaphores.push(display.present_complete_semaphore());
        };

        // TODO: this works only until we have one
        display.set_render_complete_semaphore(signal_semaphore);

        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: setup.surface_format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpass = vk::SubpassDescription::default()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = vk::RenderPassCreateInfo::default()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let render_pass = unsafe {
            gpu.create_render_pass(&renderpass_create_info)
                .expect("Failed to create a render pass")
        };

        let globals_buffer = unsafe {
            Self::create_globals_uniform_buffer(&gpu)
                .expect("Could not allocate globals uniform buffer")
        };

        let index_buffer = unsafe {
            Self::create_index_buffer(&gpu, setup.index_buffer_size)
                .expect("Could not allocate index buffer")
        };
        let vertex_buffer_non_rigged = unsafe {
            Self::create_vertex_buffer(&gpu, setup.vertex_buffer_non_rigged_size)
                .expect("Could not allocate vertex buffer (non-rigged)")
        };
        let vertex_buffer_rigged = unsafe {
            Self::create_vertex_buffer(&gpu, setup.vertex_buffer_rigged_size)
                .expect("Could not allocate vertex buffer (rigged)")
        };
        let indirect_buffer_non_rigged = unsafe {
            Self::create_indirect_buffer(&gpu, setup.indirect_buffer_non_rigged_size)
                .expect("Could not allocate indirect buffer (non-rigged)")
        };
        let indirect_buffer_rigged = unsafe {
            Self::create_indirect_buffer(&gpu, setup.indirect_buffer_rigged_size)
                .expect("Could not allocate indirect buffer (rigged)")
        };
        let instances_buffer_non_rigged = unsafe {
            Self::create_storage_buffer(&gpu, setup.instances_buffer_non_rigged_size)
                .expect("Could not allocate instances storage buffer (non-rigged)")
        };
        let instances_buffer_rigged = unsafe {
            Self::create_storage_buffer(&gpu, setup.instances_buffer_rigged_size)
                .expect("Could not allocate instances storage buffer (rigged)")
        };
        let materials_buffer = unsafe {
            Self::create_storage_buffer(&gpu, setup.materials_buffer_size)
                .expect("Could not allocate material storage buffer")
        };

        let shader_vertex_non_rigged = unsafe {
            Self::load_shader_module(&gpu, include_bytes!("shaders/non-rigged.vert.spv"))
                .expect("Failed to load non-rigged vertex shader module")
        };
        let shader_fragment_non_rigged = unsafe {
            Self::load_shader_module(&gpu, include_bytes!("shaders/non-rigged.frag.spv"))
                .expect("Failed to load non-rigged fragment shader module")
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
            // Materials
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            }, // vk::DescriptorPoolSize {
               //    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
               //    descriptor_count: 1,
               // },
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
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 2,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            // Materials
            //vk::DescriptorSetLayoutBinding {
            //    binding: 1,
            //    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            //    descriptor_count: 1,
            //    stage_flags: vk::ShaderStageFlags::FRAGMENT,
            //    ..Default::default()
            //},
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
            buffer: instances_buffer_non_rigged.handle,
            offset: 0,
            range: instances_buffer_non_rigged.size,
        };

        let material_storage_buffer_descriptor = vk::DescriptorBufferInfo {
            buffer: materials_buffer.handle,
            offset: 0,
            range: materials_buffer.size,
        };

        // let tex_descriptor = vk::DescriptorImageInfo {
        //    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        //    image_view: tex_image_view,
        //    sampler,
        // };

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
            vk::WriteDescriptorSet {
                dst_binding: 2,
                dst_set: descriptor_sets[0],
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                p_buffer_info: &material_storage_buffer_descriptor,
                ..Default::default()
            },
            //vk::WriteDescriptorSet {
            //    dst_set: descriptor_sets[0],
            //    dst_binding: 1,
            //    descriptor_count: 1,
            //    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            //    p_image_info: &tex_descriptor,
            //    ..Default::default()
            //},
        ];

        unsafe {
            gpu.update_descriptor_sets(&write_desc_sets, &[]);
        };

        // pipeline layout
        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&desc_set_layouts);
        let pipeline_layout_render_non_rigged = unsafe {
            gpu.create_pipeline_layout(&pipeline_layout_create_info)
                .expect("Failed to create non-rigged pipeline layout")
        };

        let vertex_buffer_non_rigged_size = vertex_buffer_non_rigged.size;

        Self {
            gpu,
            command_pool,
            wait_semaphores,
            signal_semaphore,
            command_buffer_setup,
            command_buffer_draw,
            command_buffer_setup_reuse_fence,
            command_buffer_draw_reuse_fence,
            render_pass,
            framebuffers: Vec::new(),
            surface_version: 0,
            index_buffer,
            index_buffer_used_size: 0,
            indirect_buffer_non_rigged,
            indirect_buffer_rigged,
            instances_buffer_non_rigged,
            instances_buffer_non_rigged_data: Vec::new(),
            instances_buffer_rigged,
            instances_buffer_rigged_data: Vec::new(),
            instances_non_rigged: HashMap::new(),
            instances_rigged: HashMap::new(),
            vertex_buffer_non_rigged,
            vertex_buffer_rigged,
            globals_buffer,
            materials_buffer,
            materials_buffer_index: HashMap::new(),
            materials_buffer_data: Vec::new(),
            mesh_layout_non_rigged: BufferLayout::new(vertex_buffer_non_rigged_size),
            shader_vertex_non_rigged,
            shader_fragment_non_rigged,
            descriptor_pool,
            desc_set_layouts,
            descriptor_sets,
            pipeline_layout_render_non_rigged,
            pipeline_render_non_rigged: vk::Pipeline::null(),
        }
    }

    pub fn globals_uniform(&self) -> GlobalsUniform {
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 800.0 / 600.0, 1.0, 10.0);
        let view = Mat4::look_at_rh(Vec3::new(1.5f32, -5.0, 3.0), Vec3::ZERO, Vec3::Z);
        GlobalsUniform {
            proj: proj.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
        }
    }

    pub unsafe fn signal_semaphore(&self) -> vk::Semaphore {
        self.signal_semaphore
    }

    fn update_buffers(&mut self, assets: &Assets, world: &World) -> u32 {
        self.instances_rigged.clear();
        self.instances_non_rigged.clear();
        self.instances_buffer_rigged_data.clear();
        self.instances_buffer_non_rigged_data.clear();
        self.materials_buffer_data.clear();

        let globals_uniform = [self.globals_uniform()];

        unsafe {
            self.globals_buffer
                .map_and_write_to_device_memory(&self.gpu, 0, &globals_uniform);
        }

        for (entity_id, mesh_id, material_id, armature_id, transform) in world.query::<(
            &Id<Entity>,
            &Id<Mesh>,
            &Id<Material>,
            &Id<Armature>,
            &Transform,
        )>() {
            log::debug!("Update buffers: {:?}", entity_id);
            let mesh_vertex_layout =
                if let Some(mesh_vertex_layout) = self.register_mesh(*mesh_id, assets) {
                    mesh_vertex_layout
                } else {
                    continue;
                };
            let material_index =
                if let Some(material_index) = self.register_material(*material_id, assets) {
                    material_index
                } else {
                    continue;
                };

            log::debug!("material index: {} ({:?})", material_index, material_id);

            self.instances_non_rigged
                .entry(*mesh_id)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(InstanceUniform {
                    transform: transform.matrix().to_cols_array_2d(),
                    material_index,
                    _padding: Default::default(),
                });
        }

        let instances_count = self.instances_non_rigged.values().map(|i| i.len()).sum();
        let draw_count = self.instances_non_rigged.len();
        let mut instances_buffer_data = Vec::with_capacity(instances_count);
        let indirect_buffer_data = self
            .instances_non_rigged
            .drain()
            .map(|(mesh_id, instances)| {
                let first_instance = instances_buffer_data.len() as u32;
                let mesh_instances_count = instances.len() as u32;
                let mesh_layout = self.mesh_layout_non_rigged.get(mesh_id).unwrap();
                instances_buffer_data.extend(instances.into_iter());

                vk::DrawIndexedIndirectCommand {
                    index_count: mesh_layout.vertices.index_count,
                    instance_count: mesh_instances_count,
                    first_instance,
                    first_index: mesh_layout.vertices.base_index,
                    vertex_offset: mesh_layout.vertices.base_vertex as i32,
                }
            })
            .collect::<Vec<_>>();

        unsafe {
            self.indirect_buffer_non_rigged
                .map_and_write_to_device_memory(&self.gpu, 0, indirect_buffer_data.as_slice());
            self.materials_buffer.map_and_write_to_device_memory(
                &self.gpu,
                0,
                self.materials_buffer_data.as_slice(),
            );
            self.instances_buffer_non_rigged
                .map_and_write_to_device_memory(&self.gpu, 0, instances_buffer_data.as_slice());
        };

        log::debug!("Instances: {:?}", instances_buffer_data);

        draw_count as u32

        /*
        let vertices = [
            (
                VertexPosition {
                    value: Vec3::new(-1.0, 1.0, 0.0),
                },
                VertexNormal {
                    value: Vec3::new(0.0, 1.0, 0.0),
                },
                VertexTexture { u: 0.0, v: 0.0 },
            ),
            (
                VertexPosition {
                    value: Vec3::new(1.0, 1.0, 0.0),
                },
                VervýhrávátexNormal {
                    value: Vec3::new(0.0, 0.0, 1.0),
                },
                VertexTexture { u: 0.0, v: 0.0 },
            ),
            (
                VertexPosition {
                    value: Vec3::new(0.0, -1.0, 0.0),
                },
                VertexNormal {
                    value: Vec3::new(1.0, 0.0, 0.0),
                },
                VertexTexture { u: 0.0, v: 0.0 },
            ),
        ];


        let draw_indirect_command = [DrawIndirectCommand {
            vertex_count: 3,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
        }];

        unsafe {
            Self::map_and_write_to_device_memory(
                &self.gpu,
                self.buffer_vertex_memory,
                0,
                &vertices,
            );
            Self::map_and_write_to_device_memory(
                &self.gpu,
                self.buffer_indirect_memory,
                0,
                &draw_indirect_command,
            );
        };
         */
    }

    fn register_mesh(&mut self, mesh_id: Id<Mesh>, assets: &Assets) -> Option<MeshVerticesLayout> {
        // check if the mesh is already in buffer
        if let Some(mesh_layout) = self.mesh_layout_non_rigged.get(mesh_id) {
            return Some(mesh_layout.vertices.clone());
        }
        // try to get mesh to store it in buffer
        if let Some(mesh) = assets.get(mesh_id) {
            if mesh.indices::<u8>().is_some() {
                panic!("Index buffer is not implemented yet");
            }
            if let Some(vertex_data) = mesh.buffer::<VertexBufferLayoutNonRigged>() {
                use crate::models::meshes::VertexBufferLayout;
                let vertex_size = VertexBufferLayoutNonRigged::vertex_size() as u64;
                let data_size = vertex_data.len() as u64;
                let vertex_offset = self.mesh_layout_non_rigged.used_size();
                let vertex_count = mesh.count_vertices() as u32;
                let index_offset =
                    (self.index_buffer_used_size / (std::mem::size_of::<u32>() as u64)) as u32;

                let mut auto_indices = Vec::new();
                let index_data = mesh.indices::<u32>().unwrap_or_else(|| {
                    auto_indices = (0..vertex_count).collect::<Vec<u32>>();
                    auto_indices.as_slice()
                });
                let index_count = index_data.len() as u32;
                let index_data_size = (index_data.len() * std::mem::size_of::<u32>()) as u64;

                let vertex_layout = MeshVerticesLayout {
                    base_vertex: (vertex_offset / vertex_size) as u32,
                    vertex_count: mesh.count_vertices() as u32,
                    base_index: index_offset,
                    index_count,
                };

                self.mesh_layout_non_rigged
                    .store(
                        mesh_id,
                        MeshLayout {
                            in_vertex_buffer: LayoutInBuffer {
                                offset: vertex_offset,
                                size: data_size,
                            },
                            in_index_buffer: LayoutInBuffer {
                                offset: self.index_buffer_used_size,
                                size: index_data_size,
                            },
                            vertices: vertex_layout.clone(),
                        },
                        data_size,
                    )
                    .ok();

                unsafe {
                    self.vertex_buffer_non_rigged
                        .map_and_write_to_device_memory(
                            &self.gpu,
                            vertex_offset,
                            vertex_data.as_slice(),
                        );

                    log::debug!("Indices @{}: {:?}", self.index_buffer_used_size, index_data);
                    self.index_buffer.map_and_write_to_device_memory(
                        &self.gpu,
                        self.index_buffer_used_size,
                        index_data,
                    );
                };

                self.index_buffer_used_size += index_data_size;

                return Some(vertex_layout);
            } else {
                log::error!("Could not store the mesh named `{}`", mesh.name());
            }
        }
        None
    }

    fn register_material(&mut self, material_id: Id<Material>, assets: &Assets) -> Option<u32> {
        assets.get(material_id).and_then(|material| {
            self.materials_buffer_index
                .get(&material_id)
                .cloned()
                .or_else(|| {
                    let index = self.materials_buffer_index.len() as u32;
                    self.materials_buffer_data.push(MaterialUniform::default());
                    self.materials_buffer_index.insert(material_id, index);
                    Some(index)
                })
                .map(|material_index| {
                    self.materials_buffer_data[material_index as usize] = material.into();
                    material_index
                })
        })
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

    unsafe fn create_indirect_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
        let buffer_create_info = vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::INDIRECT_BUFFER,
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

    unsafe fn load_shader_module(gpu: &Gpu, bytes: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
        // let bytes = include_bytes!("shaders/non-rigged.frag.spv");
        let mut cursor = Cursor::new(&bytes[..]);
        let shader_code = ash::util::read_spv(&mut cursor).expect("Failed to read shader SPV code");
        let shader_module_create_info = vk::ShaderModuleCreateInfo::default().code(&shader_code);

        gpu.create_shader_module(&shader_module_create_info)
    }

    unsafe fn create_framebuffers(&mut self, display: &Display, render_pass: vk::RenderPass) {
        let resolution = display.surface_resolution();
        self.framebuffers = display
            .swapchain_image_views()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, display.depth_image_view()];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&framebuffer_attachments)
                    .width(resolution.width)
                    .height(resolution.height)
                    .layers(1);

                self.gpu
                    .create_framebuffer(&frame_buffer_create_info)
                    .expect("Could not create a framebuffer")
            })
            .collect::<Vec<_>>()
    }

    unsafe fn destroy_framebuffers(&mut self) {
        for framebuffer in self.framebuffers.drain(..) {
            self.gpu.destroy_framebuffer(framebuffer);
        }
    }

    unsafe fn create_graphics_pipelines(&self, surface_resolution: Extent2D) -> vk::Pipeline {
        let shader_entry_point = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                module: self.shader_vertex_non_rigged,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: self.shader_fragment_non_rigged,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        // vertex binding
        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<VertexBufferLayoutNonRigged>() as u32,
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
            // texture
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
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
            polygon_mode: vk::PolygonMode::FILL,
            ..Default::default()
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
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
            .layout(self.pipeline_layout_render_non_rigged)
            .render_pass(self.render_pass);

        let graphics_pipelines = self
            .gpu
            .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info])
            .expect("Failed to create graphics pipelines");

        let graphic_pipeline = graphics_pipelines[0];

        graphic_pipeline
    }

    unsafe fn destroy_graphics_pipelines(&self) {
        self.gpu.destroy_pipeline(self.pipeline_render_non_rigged);
    }

    unsafe fn execute_render_pass(&self, frame: &Frame, draw_count: u32) {
        self.gpu
            .wait_for_fences(&[self.command_buffer_draw_reuse_fence], true, u64::MAX)
            .expect("Failed to wait for draw buffer fences");

        self.gpu
            .reset_fences(&[self.command_buffer_draw_reuse_fence])
            .expect("Failed to reset Vulkan fences");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.gpu
            .reset_command_buffer(
                self.command_buffer_draw,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Failed to reset Vulkan command buffer");

        self.gpu
            .begin_command_buffer(self.command_buffer_draw, &command_buffer_begin_info)
            .expect("Failed to begin draw command buffer");

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.1, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: frame.resolution.width as f32,
            height: frame.resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [(vk::Extent2D {
            width: frame.resolution.width,
            height: frame.resolution.height,
        })
        .into()];

        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[frame.swapchain_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: frame.resolution.width,
                    height: frame.resolution.height,
                },
            })
            .clear_values(&clear_values);

        self.gpu.cmd_begin_render_pass(
            self.command_buffer_draw,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        );

        self.gpu.cmd_bind_descriptor_sets(
            self.command_buffer_draw,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout_render_non_rigged,
            0,
            &self.descriptor_sets[..],
            &[],
        );

        self.gpu.cmd_bind_pipeline(
            self.command_buffer_draw,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_render_non_rigged,
        );
        self.gpu
            .cmd_set_viewport(self.command_buffer_draw, 0, &viewports);
        self.gpu
            .cmd_set_scissor(self.command_buffer_draw, 0, &scissors);
        self.gpu.cmd_bind_vertex_buffers(
            self.command_buffer_draw,
            0,
            &[self.vertex_buffer_non_rigged.handle],
            &[0],
        );

        self.gpu.cmd_bind_index_buffer(
            self.command_buffer_draw,
            self.index_buffer.handle,
            0,
            vk::IndexType::UINT32,
        );

        // TODO: draw indirect
        self.gpu.cmd_draw_indexed_indirect(
            self.command_buffer_draw,
            self.indirect_buffer_non_rigged.handle,
            0,
            draw_count,
            0, //std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u32,
        );

        self.gpu.cmd_end_render_pass(self.command_buffer_draw);

        self.gpu
            .end_command_buffer(self.command_buffer_draw)
            .expect("End commandbuffer");

        // submit
        /*

        self.gpu
            .end_command_buffer(self.command_buffer_draw)
            .expect("End commandbuffer");

        let command_buffers = [self.command_buffer_draw];
        let wait_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.signal_semaphore];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&self.wait_semaphores)
            .wait_dst_stage_mask(&wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        self.gpu
            .submit_queue(&[submit_info], self.command_buffer_draw_reuse_fence)
            .expect("queue submit failed.");
        */
    }

    unsafe fn setup_depth_image(&self, display: &Display) {
        let depth_image = display.depth_image();

        // begin: prepare

        self.gpu
            .wait_for_fences(&[self.command_buffer_setup_reuse_fence], true, u64::MAX)
            .expect("Wait for fence failed.");

        self.gpu
            .reset_fences(&[self.command_buffer_setup_reuse_fence])
            .expect("Reset fences failed.");

        self.gpu
            .reset_command_buffer(
                self.command_buffer_setup,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Reset command buffer failed.");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.gpu
            .begin_command_buffer(self.command_buffer_setup, &command_buffer_begin_info)
            .expect("Begin commandbuffer");

        // end: prepare

        let layout_transition_barriers = vk::ImageMemoryBarrier::default()
            .image(depth_image)
            .dst_access_mask(
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .layer_count(1)
                    .level_count(1),
            );

        self.gpu.cmd_pipeline_barrier(
            self.command_buffer_setup,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[layout_transition_barriers],
        );

        // submit
        self.gpu
            .end_command_buffer(self.command_buffer_setup)
            .expect("End commandbuffer");

        /*
        let command_buffers = [self.command_buffer_setup];
        let wait_mask = [];
        let wait_semaphores = [];
        let signal_semaphores = [];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        self.gpu
            .queue_submit(&[submit_info], self.command_buffer_setup_reuse_fence)
            .expect("queue submit failed.")
             */
    }

    unsafe fn submit_draw_commands(&self) {
        let (wait_semaphores, wait_dst_stage_mask): (Vec<_>, Vec<_>) = self
            .wait_semaphores
            .iter()
            .map(|s| (s, vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT))
            .unzip();
        let signal_semaphores = [self.signal_semaphore];
        let command_buffers = [self.command_buffer_draw];
        let submits = [vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores.as_slice())
            .wait_dst_stage_mask(wait_dst_stage_mask.as_slice())
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)];

        log::debug!(
            "buffers: {}, wait: {}, signal: {}, deps: {}",
            command_buffers.len(),
            wait_semaphores.len(),
            signal_semaphores.len(),
            wait_dst_stage_mask.len(),
        );
        self.gpu
            .submit_queue(&submits, self.command_buffer_draw_reuse_fence)
            .expect("Failed to submit draw buffer to queue");
    }
}

pub struct RenderModelsSetup {
    wait_semaphores: Vec<vk::Semaphore>,
    surface_format: vk::Format,
    index_buffer_size: u64,
    vertex_buffer_non_rigged_size: u64,
    vertex_buffer_rigged_size: u64,
    indirect_buffer_non_rigged_size: u64,
    indirect_buffer_rigged_size: u64,
    instances_buffer_non_rigged_size: u64,
    instances_buffer_rigged_size: u64,
    materials_buffer_size: u64,
}

impl Default for RenderModelsSetup {
    fn default() -> Self {
        Self {
            wait_semaphores: Vec::new(),
            surface_format: vk::Format::default(),
            index_buffer_size: 8 * 1024 * 1024,
            vertex_buffer_non_rigged_size: 8 * 1024 * 1024,
            vertex_buffer_rigged_size: 8 * 1024 * 1024,
            indirect_buffer_non_rigged_size: 1000
                * std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64,
            indirect_buffer_rigged_size: 1000
                * std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64,
            instances_buffer_non_rigged_size: 1000 * std::mem::size_of::<InstanceUniform>() as u64,
            instances_buffer_rigged_size: 1000 * std::mem::size_of::<InstanceUniform>() as u64,
            materials_buffer_size: 1000 * std::mem::size_of::<MaterialUniform>() as u64,
        }
    }
}

impl RenderModelsSetup {
    pub fn wait_semaphores(mut self, semaphores: impl IntoIterator<Item = vk::Semaphore>) -> Self {
        self.wait_semaphores.extend(semaphores);
        self
    }

    pub fn surface_format(mut self, surface_format: vk::Format) -> Self {
        self.surface_format = surface_format;
        self
    }

    pub fn create(self, display: &mut Display) -> RenderModels {
        RenderModels::new(display, self)
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
    /// Model transform matrix
    pub transform: [[f32; 4]; 4],
    /// material index in buffer
    pub material_index: u32,
    /// padding
    pub _padding: [u32; 3],
}
