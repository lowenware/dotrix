use std::collections::HashMap;
use std::ffi::CStr;
use std::io::Cursor;

use crate::graphics::vk;
use crate::graphics::{Buffer, RenderPass};
use crate::loaders::Assets;
use crate::models::materials::MAX_MATERIAL_IMAGES;
use crate::utils::Id;
use crate::world::{Camera, Entity, World};
use crate::{log, VertexJoints, VertexWeights};
use crate::{Any, Asset, Display, Extent2D, Frame, Gpu, Image, Ref, Task};

use super::materials::MaterialUniform;
use super::{
    Armature, Material, Mesh, Transform, VertexBufferLayout, VertexNormal, VertexPosition,
    VertexTexture,
};

#[derive(Clone, Copy)]
pub struct LayoutInBuffer {
    /// Offset inside of the buffer in bytes
    pub offset: u64,
    /// Offset of the first item (vertex or index)
    pub base: u32,
    /// Number of items (vertices or indices)
    pub count: u32,
}

/// Layout of a single mesh in buffers
#[derive(Clone, Copy)]
pub struct MeshLayout {
    pub vertices: LayoutInBuffer,
    pub indices: Option<LayoutInBuffer>,
    pub has_skin: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct DrawCount {
    pub only_mesh: u32,
    pub only_mesh_indexed: u32,
    pub skin_mesh: u32,
    pub skin_mesh_indexed: u32,
}

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
    /// Index of instances by mesh (just mesh, indexed)
    instances_only_mesh_indexed: HashMap<Id<Mesh>, Vec<InstanceUniform>>,
    /// Index of instances by mesh (with skin, indexed)
    instances_skin_mesh_indexed: HashMap<Id<Mesh>, Vec<InstanceUniform>>,
    /// Index of instances by mesh (just mesh, not indexed)
    instances_only_mesh: HashMap<Id<Mesh>, Vec<InstanceUniform>>,
    /// Index of instances by mesh (with skin, not indexed)
    instances_skin_mesh: HashMap<Id<Mesh>, Vec<InstanceUniform>>,
    /// Transform buffer content
    transform_buffer_data: Vec<TransformUniform>,
    /// Globals uniform buffer
    globals_buffer: Buffer,
    /// Indices buffer (rigged and non-rigged)
    index_buffer: Buffer,
    /// Used bytes in index buffer,
    index_buffer_usage: u64,
    /// Vertex buffer (non-rigged)
    vertex_buffer_only_mesh: Buffer,
    vertex_buffer_only_mesh_usage: u64,
    /// Vertex buffer (rigged)
    vertex_buffer_skin_mesh: Buffer,
    vertex_buffer_skin_mesh_usage: u64,
    /// Indstance buffer
    instance_buffer: Buffer,
    /// Transforms (model and joints) buffer
    transform_buffer: Buffer,
    /// Indirect buffer
    indirect_buffer: Buffer,
    /// Materials buffer
    material_buffer: Buffer,
    /// Mapping of material index in the buffer by its ID
    material_buffer_index: HashMap<Id<Material>, u32>,
    /// Materials buffer data
    material_buffer_data: Vec<MaterialUniform>,
    /// Material sampler
    material_sampler: vk::Sampler,
    /// Material image view
    material_image_view: vk::ImageView,
    /// Material image memory
    material_image_memory: vk::DeviceMemory,
    /// Material image
    material_image: vk::Image,
    /// Buffer to copy material images to material_image
    material_staging_buffer: Buffer,
    /// Number of layers in material image buffer
    material_layer_count: u32,
    material_layer_size: Extent2D,
    /// Material layer index in the material_image
    material_layer_index: HashMap<Id<Image>, usize>,
    /// Usage of layers in material image buffer
    material_layer_usage: u32,
    /// Mesh layouts of non-rigged models
    mesh_registry: HashMap<Id<Mesh>, MeshLayout>,
    /// descriptor sets
    descriptor_sets: Vec<vk::DescriptorSet>,
    /// descriptor pool
    descriptor_pool: vk::DescriptorPool,
    /// descriptor set layouts
    desc_set_layouts: [vk::DescriptorSetLayout; 1],
    /// Pipeline layout to render only mesh models
    pipeline_layout_render: vk::PipelineLayout,
    /// Graphics pipeline to render only mesh models
    pipeline_render_only_mesh: vk::Pipeline,
    /// Vertex shader module for only mesh pipeline
    shader_vertex_only_mesh: vk::ShaderModule,
    /// Fragment shader module for only mesh pipeline
    shader_fragment_only_mesh: vk::ShaderModule,
    /// Graphics pipeline to render skin mesh models
    pipeline_render_skin_mesh: vk::Pipeline,
    /// Vertex shader module for skin mesh pipeline
    shader_vertex_skin_mesh: vk::ShaderModule,
    /// Fragment shader module for skin mesh pipeline
    shader_fragment_skin_mesh: vk::ShaderModule,
}

pub type VertexBufferOnlyMeshLayout = (VertexPosition, VertexNormal, VertexTexture);
pub type VertexBufferSkinMeshLayout = (
    VertexPosition,
    VertexNormal,
    VertexTexture,
    VertexWeights,
    VertexJoints,
);

impl Drop for RenderModels {
    fn drop(&mut self) {
        unsafe {
            self.gpu.device_wait_idle().unwrap();

            // pipelines
            self.destroy_graphics_pipelines();

            // pipelines layouts
            self.gpu
                .destroy_pipeline_layout(self.pipeline_layout_render);

            // shaders
            self.gpu.destroy_shader_module(self.shader_vertex_only_mesh);
            self.gpu
                .destroy_shader_module(self.shader_fragment_only_mesh);
            self.gpu.destroy_shader_module(self.shader_vertex_skin_mesh);
            self.gpu
                .destroy_shader_module(self.shader_fragment_skin_mesh);

            // buffers
            self.globals_buffer.free_memory_and_destroy(&self.gpu);
            self.index_buffer.free_memory_and_destroy(&self.gpu);
            self.vertex_buffer_only_mesh
                .free_memory_and_destroy(&self.gpu);
            self.vertex_buffer_skin_mesh
                .free_memory_and_destroy(&self.gpu);
            self.indirect_buffer.free_memory_and_destroy(&self.gpu);
            self.instance_buffer.free_memory_and_destroy(&self.gpu);
            self.material_buffer.free_memory_and_destroy(&self.gpu);
            self.transform_buffer.free_memory_and_destroy(&self.gpu);
            self.material_staging_buffer
                .free_memory_and_destroy(&self.gpu);

            // samplers
            self.gpu.destroy_sampler(self.material_sampler);

            // image views
            self.gpu.destroy_image_view(self.material_image_view);

            // images and memory
            self.gpu.free_memory(self.material_image_memory);
            self.gpu.destroy_image(self.material_image);

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
    type Context = (
        Any<Frame>,
        Any<Camera>,
        Ref<Assets>,
        Ref<Display>,
        Ref<World>,
    );
    type Output = RenderPass;

    fn run(&mut self, (frame, camera, assets, display, world): Self::Context) -> Self::Output {
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
                if self.pipeline_render_only_mesh == vk::Pipeline::null() {
                    log::debug!("resize: destroy_graphics_pipelines");
                    self.destroy_graphics_pipelines();
                    log::debug!("resize: create_graphics_pipelines");
                    let graphic_pipelines =
                        self.create_graphics_pipelines(display.surface_resolution());
                    self.pipeline_render_only_mesh = graphic_pipelines[0];
                    self.pipeline_render_skin_mesh = graphic_pipelines[1];

                    // NOTE: the setup buffer should be probably a part of the Display
                    log::debug!("resize: setup_depth_image");
                    self.setup_depth_image(&display);
                }

                log::debug!("resize: complete -> {}", surface_version);
            };
            self.surface_version = surface_version;
        }

        let draw_count = self.update_buffers(&camera, &assets, &world);

        log::debug!("draw count: {:?}", draw_count);

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
        let vertex_buffer_only_mesh = unsafe {
            Self::create_vertex_buffer(&gpu, setup.vertex_buffer_only_mesh_size)
                .expect("Could not allocate vertex buffer (non-rigged)")
        };
        let vertex_buffer_skin_mesh = unsafe {
            Self::create_vertex_buffer(&gpu, setup.vertex_buffer_skin_mesh_size)
                .expect("Could not allocate vertex buffer (rigged)")
        };
        let indirect_buffer = unsafe {
            Self::create_indirect_buffer(&gpu, setup.indirect_buffer_size)
                .expect("Could not allocate indirect buffer")
        };
        let material_buffer = unsafe {
            Self::create_storage_buffer(&gpu, setup.material_buffer_size)
                .expect("Could not allocate material storage buffer")
        };
        let instance_buffer = unsafe {
            Self::create_storage_buffer(&gpu, setup.instance_buffer_size)
                .expect("Could not allocate instances storage buffer")
        };
        let transform_buffer = unsafe {
            Self::create_storage_buffer(&gpu, setup.transform_buffer_size)
                .expect("Could not allocate transform storage buffer")
        };

        let shader_vertex_only_mesh = unsafe {
            Self::load_shader_module(&gpu, include_bytes!("shaders/only_mesh.vert.spv"))
                .expect("Failed to load only-mesh vertex shader module")
        };
        let shader_fragment_only_mesh = unsafe {
            Self::load_shader_module(&gpu, include_bytes!("shaders/only_mesh.frag.spv"))
                .expect("Failed to load only-mesh fragment shader module")
        };
        let shader_vertex_skin_mesh = unsafe {
            Self::load_shader_module(&gpu, include_bytes!("shaders/skin_mesh.vert.spv"))
                .expect("Failed to load skin-mesh vertex shader module")
        };
        let shader_fragment_skin_mesh = unsafe {
            Self::load_shader_module(&gpu, include_bytes!("shaders/skin_mesh.frag.spv"))
                .expect("Failed to load skin-mesh fragment shader module")
        };
        let material_sampler_create_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::MIRRORED_REPEAT,
            address_mode_v: vk::SamplerAddressMode::MIRRORED_REPEAT,
            address_mode_w: vk::SamplerAddressMode::MIRRORED_REPEAT,
            max_anisotropy: 1.0,
            border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
            compare_op: vk::CompareOp::NEVER,
            ..Default::default()
        };

        let material_sampler = unsafe {
            gpu.create_sampler(&material_sampler_create_info)
                .expect("Failed to create materials sampler")
        };

        let material_layer_count = setup.material_count * MAX_MATERIAL_IMAGES;

        let material_image_create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_UNORM,
            extent: vk::Extent3D {
                width: setup.material_image_size.width,
                height: setup.material_image_size.height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: material_layer_count,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let material_image = unsafe {
            gpu.create_image(&material_image_create_info)
                .expect("Failed to create material image")
        };
        let material_image_memory_req =
            unsafe { gpu.get_image_memory_requirements(material_image) };
        let material_image_memory_index = gpu
            .find_memory_type_index(
                &material_image_memory_req,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .expect("Unable to find suitable memory index for depth image.");

        let material_image_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: material_image_memory_req.size,
            memory_type_index: material_image_memory_index,
            ..Default::default()
        };
        let material_image_memory = unsafe {
            gpu.allocate_memory(&material_image_allocate_info)
                .expect("Failed to allocate material image")
        };
        unsafe {
            gpu.bind_image_memory(material_image, material_image_memory, 0)
                .expect("Unable to bind depth image memory");
        };

        let material_image_view_info = vk::ImageViewCreateInfo {
            view_type: vk::ImageViewType::TYPE_2D_ARRAY,
            format: material_image_create_info.format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: material_layer_count,
                ..Default::default()
            },
            image: material_image,
            ..Default::default()
        };
        let material_image_view = unsafe {
            gpu.create_image_view(&material_image_view_info)
                .expect("Failed to create material image view")
        };

        let material_staging_buffer_size = MAX_MATERIAL_IMAGES
            * setup.material_image_size.width
            * setup.material_image_size.height
            * std::mem::size_of::<u32>() as u32;

        let material_staging_buffer = unsafe {
            Self::create_transfer_buffer(&gpu, material_staging_buffer_size as u64)
                .expect("Could not allocate material staging buffer")
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
            // Material
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            },
            // Transform
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            },
            // Materials textures sampler
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
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
            // Material
            vk::DescriptorSetLayoutBinding {
                binding: 2,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
            // Transform
            vk::DescriptorSetLayoutBinding {
                binding: 3,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            // Materials textures sampler
            vk::DescriptorSetLayoutBinding {
                binding: 4,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
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

        let material_storage_buffer_descriptor = vk::DescriptorBufferInfo {
            buffer: material_buffer.handle,
            offset: 0,
            range: material_buffer.size,
        };

        let transform_storage_buffer_descriptor = vk::DescriptorBufferInfo {
            buffer: transform_buffer.handle,
            offset: 0,
            range: transform_buffer.size,
        };
        let tex_descriptor = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: material_image_view,
            sampler: material_sampler,
        };

        let only_mesh_write_desc_sets = [
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
            vk::WriteDescriptorSet {
                dst_binding: 3,
                dst_set: descriptor_sets[0],
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                p_buffer_info: &transform_storage_buffer_descriptor,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_binding: 4,
                dst_set: descriptor_sets[0],
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                p_image_info: &tex_descriptor,
                ..Default::default()
            },
        ];

        unsafe {
            gpu.update_descriptor_sets(&only_mesh_write_desc_sets, &[]);
        };

        // pipeline layout
        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&desc_set_layouts);
        let pipeline_layout_render = unsafe {
            gpu.create_pipeline_layout(&pipeline_layout_create_info)
                .expect("Failed to create non-rigged pipeline layout")
        };

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
            index_buffer_usage: 0,
            indirect_buffer,
            instance_buffer,
            instances_only_mesh_indexed: HashMap::new(),
            instances_skin_mesh_indexed: HashMap::new(),
            instances_only_mesh: HashMap::new(),
            instances_skin_mesh: HashMap::new(),
            vertex_buffer_only_mesh,
            vertex_buffer_only_mesh_usage: 0,
            vertex_buffer_skin_mesh_usage: 0,
            vertex_buffer_skin_mesh,
            globals_buffer,
            material_buffer,
            transform_buffer_data: Vec::new(),
            transform_buffer,
            material_buffer_index: HashMap::new(),
            material_buffer_data: Vec::new(),
            material_sampler,
            material_image_view,
            material_image,
            material_image_memory,
            material_staging_buffer,
            material_layer_count,
            material_layer_usage: 0,
            material_layer_index: HashMap::new(),
            material_layer_size: setup.material_image_size,
            mesh_registry: HashMap::new(),
            descriptor_pool,
            desc_set_layouts,
            descriptor_sets,
            pipeline_layout_render,
            pipeline_render_only_mesh: vk::Pipeline::null(),
            shader_vertex_only_mesh,
            shader_fragment_only_mesh,
            pipeline_render_skin_mesh: vk::Pipeline::null(),
            shader_vertex_skin_mesh,
            shader_fragment_skin_mesh,
        }
    }

    pub fn globals_uniform(&self, camera: &Camera) -> GlobalsUniform {
        // let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 800.0 / 600.0, 1.0, 10.0);
        // let view = Mat4::look_at_rh(Vec3::new(1.5f32, -5.0, 3.0), Vec3::ZERO, Vec3::Z);
        GlobalsUniform {
            proj: camera.proj.to_cols_array_2d(),
            view: camera.view.to_cols_array_2d(),
        }
    }

    /// # Safety
    ///
    /// Leaks signal semaphore that should never be destroyed
    pub unsafe fn signal_semaphore(&self) -> vk::Semaphore {
        self.signal_semaphore
    }

    fn update_buffers(&mut self, camera: &Camera, assets: &Assets, world: &World) -> DrawCount {
        self.instances_skin_mesh_indexed.clear();
        self.instances_only_mesh_indexed.clear();
        self.instances_skin_mesh.clear();
        self.instances_only_mesh.clear();
        self.transform_buffer_data.clear();
        // self.material_buffer_data.clear();

        let globals_uniform = [self.globals_uniform(camera)];

        unsafe {
            self.globals_buffer
                .map_and_write_to_device_memory(&self.gpu, 0, &globals_uniform);
        }

        for (entity_id, mesh_id, material_id, _armature_id, transform) in world.query::<(
            &Id<Entity>,
            &Id<Mesh>,
            &Id<Material>,
            &Id<Armature>,
            &Transform,
        )>() {
            log::debug!("Update buffers: {:?}", entity_id);
            let material_index = self.register_material(*material_id, assets);
            if material_index.is_none() {
                continue;
            }
            let material_index = material_index.unwrap();

            if let Some(mesh_layout) = self.register_mesh(*mesh_id, assets) {
                let instances = if mesh_layout.has_skin {
                    if mesh_layout.indices.is_some() {
                        &mut self.instances_skin_mesh_indexed
                    } else {
                        &mut self.instances_skin_mesh
                    }
                } else if mesh_layout.indices.is_some() {
                    &mut self.instances_only_mesh_indexed
                } else {
                    &mut self.instances_only_mesh
                };
                let transform_index = self.transform_buffer_data.len() as u32;
                self.transform_buffer_data.push(TransformUniform {
                    transform: transform.model.matrix().to_cols_array_2d(),
                });
                let mut joint_index = 0; // NOTE: real joint can never have a 0 index
                if mesh_layout.has_skin && transform.armature.len() != 0 {
                    joint_index = self.transform_buffer_data.len() as u32;
                    self.transform_buffer_data
                        .extend(transform.armature.iter().map(|i| TransformUniform {
                            transform: i.to_cols_array_2d(),
                        }))
                };
                instances
                    .entry(*mesh_id)
                    .or_insert_with(|| Vec::with_capacity(1))
                    .push(InstanceUniform {
                        material_index,
                        transform_index,
                        joint_index,
                        ..Default::default()
                    });

                log::debug!("\n\n    material index: {} ({:?})\n    transform_index: {}\n    joint_index: {}\n\n", material_index, material_id, transform_index, joint_index);

                if mesh_layout.has_skin {}
            }
        }

        unsafe {
            let material_bytes = self.material_buffer.map_and_write_to_device_memory(
                &self.gpu,
                0,
                self.material_buffer_data.as_slice(),
            );
            log::debug!(
                "writing materials buffer ({:?}): {:?}",
                material_bytes,
                self.material_buffer_data.as_slice()
            );
        };
        unsafe {
            let transform_bytes = self.transform_buffer.map_and_write_to_device_memory(
                &self.gpu,
                0,
                self.transform_buffer_data.as_slice(),
            );
            log::debug!("writing materials buffer ({:?})", transform_bytes);
        };

        let mut instances_total = 0;
        let mut instance_buffer_offset: u64 = 0;
        let mut indirect_buffer_offset: u64 = 0;

        // No indices, no skin
        let only_mesh_draws_count = if !self.instances_only_mesh.is_empty() {
            indirect_buffer_offset +=
                indirect_buffer_offset % std::mem::size_of::<vk::DrawIndirectCommand>() as u64;
            let instances_count = self
                .instances_only_mesh
                .values()
                .map(|i| i.len() as u32)
                .sum::<u32>();
            let draw_count = self.instances_only_mesh.len();
            let mut instance_buffer_data = Vec::with_capacity(instances_count as usize);
            let indirect_buffer_data = self
                .instances_only_mesh
                .drain()
                .map(|(mesh_id, instances)| {
                    let first_instance = instances_total + instance_buffer_data.len() as u32;
                    let mesh_instances_count = instances.len() as u32;
                    let mesh_layout = self.mesh_registry.get(&mesh_id).unwrap();
                    instance_buffer_data.extend(instances);

                    vk::DrawIndirectCommand {
                        vertex_count: mesh_layout.vertices.count,
                        instance_count: mesh_instances_count,
                        first_instance,
                        first_vertex: mesh_layout.vertices.base,
                    }
                })
                .collect::<Vec<_>>();

            log::debug!(
                "only mesh: instance buffer data (offest: {}): {:?}",
                instance_buffer_offset,
                instance_buffer_data
            );
            log::debug!(
                "only mesh: indirect buffer data (offest: {}): {:?}",
                indirect_buffer_offset,
                indirect_buffer_data
            );

            instances_total += instances_count;

            unsafe {
                indirect_buffer_offset += self.indirect_buffer.map_and_write_to_device_memory(
                    &self.gpu,
                    indirect_buffer_offset,
                    indirect_buffer_data.as_slice(),
                );
                instance_buffer_offset += self.instance_buffer.map_and_write_to_device_memory(
                    &self.gpu,
                    instance_buffer_offset,
                    instance_buffer_data.as_slice(),
                );
            }
            draw_count
        } else {
            0
        };

        // With indices, no skin
        let only_mesh_indexed_draws_count = if !self.instances_only_mesh_indexed.is_empty() {
            indirect_buffer_offset += indirect_buffer_offset
                % std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64;
            let instances_count = self
                .instances_only_mesh_indexed
                .values()
                .map(|i| i.len() as u32)
                .sum::<u32>();
            let draw_count = self.instances_only_mesh_indexed.len();
            let mut instance_buffer_data = Vec::with_capacity(instances_count as usize);
            let indirect_buffer_data = self
                .instances_only_mesh_indexed
                .drain()
                .map(|(mesh_id, instances)| {
                    let first_instance = instances_total + instance_buffer_data.len() as u32;
                    let mesh_instances_count = instances.len() as u32;
                    let mesh_registry = self.mesh_registry.get(&mesh_id).unwrap();
                    instance_buffer_data.extend(instances);

                    vk::DrawIndexedIndirectCommand {
                        index_count: mesh_registry.indices.unwrap().count,
                        instance_count: mesh_instances_count,
                        first_instance,
                        first_index: mesh_registry.indices.unwrap().base,
                        vertex_offset: mesh_registry.vertices.base as i32,
                    }
                })
                .collect::<Vec<_>>();

            log::debug!(
                "only mesh(indexed): instance buffer data (offest: {}): {:?}",
                instance_buffer_offset,
                instance_buffer_data
            );
            log::debug!(
                "only mesh(indexed): indirect buffer data (offest: {}): {:?}",
                indirect_buffer_offset,
                indirect_buffer_data
            );

            instances_total += instances_count;

            unsafe {
                indirect_buffer_offset += self.indirect_buffer.map_and_write_to_device_memory(
                    &self.gpu,
                    indirect_buffer_offset,
                    indirect_buffer_data.as_slice(),
                );
                instance_buffer_offset += self.instance_buffer.map_and_write_to_device_memory(
                    &self.gpu,
                    instance_buffer_offset,
                    instance_buffer_data.as_slice(),
                );
            }
            draw_count
        } else {
            0
        };

        // No indices, with skin
        let skin_mesh_draws_count = if !self.instances_skin_mesh.is_empty() {
            indirect_buffer_offset +=
                indirect_buffer_offset % std::mem::size_of::<vk::DrawIndirectCommand>() as u64;
            log::debug!(
                "skin mesh: indirect buffer data (offest: {})",
                indirect_buffer_offset,
            );
            let instances_count = self
                .instances_skin_mesh
                .values()
                .map(|i| i.len() as u32)
                .sum::<u32>();
            let draw_count = self.instances_skin_mesh.len();
            let mut instance_buffer_data = Vec::with_capacity(instances_count as usize);
            let indirect_buffer_data = self
                .instances_skin_mesh
                .drain()
                .map(|(mesh_id, instances)| {
                    let first_instance = instances_total + instance_buffer_data.len() as u32;
                    let mesh_instances_count = instances.len() as u32;
                    let mesh_layout = self.mesh_registry.get(&mesh_id).unwrap();
                    instance_buffer_data.extend(instances);

                    vk::DrawIndirectCommand {
                        vertex_count: mesh_layout.vertices.count,
                        instance_count: mesh_instances_count,
                        first_instance,
                        first_vertex: mesh_layout.vertices.base,
                    }
                })
                .collect::<Vec<_>>();

            instances_total += instances_count;

            unsafe {
                indirect_buffer_offset += self.indirect_buffer.map_and_write_to_device_memory(
                    &self.gpu,
                    indirect_buffer_offset,
                    indirect_buffer_data.as_slice(),
                );
                instance_buffer_offset += self.instance_buffer.map_and_write_to_device_memory(
                    &self.gpu,
                    instance_buffer_offset,
                    instance_buffer_data.as_slice(),
                );
            }
            draw_count
        } else {
            0
        };

        // With indices, with skin
        let skin_mesh_indexed_draws_count = if !self.instances_skin_mesh_indexed.is_empty() {
            indirect_buffer_offset += indirect_buffer_offset
                % std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64;
            log::debug!(
                "skin mesh(indexed): indirect buffer data (offest: {})",
                indirect_buffer_offset,
            );
            let instances_count = self
                .instances_skin_mesh_indexed
                .values()
                .map(|i| i.len() as u32)
                .sum::<u32>();
            let draw_count = self.instances_skin_mesh_indexed.len();
            let mut instance_buffer_data = Vec::with_capacity(instances_count as usize);
            let indirect_buffer_data = self
                .instances_skin_mesh_indexed
                .drain()
                .map(|(mesh_id, instances)| {
                    let first_instance = instances_total + instance_buffer_data.len() as u32;
                    let mesh_instances_count = instances.len() as u32;
                    let mesh_layout = self.mesh_registry.get(&mesh_id).unwrap();
                    instance_buffer_data.extend(instances);

                    vk::DrawIndexedIndirectCommand {
                        index_count: mesh_layout.indices.unwrap().count,
                        instance_count: mesh_instances_count,
                        first_instance,
                        first_index: mesh_layout.indices.unwrap().base,
                        vertex_offset: mesh_layout.vertices.base as i32,
                    }
                })
                .collect::<Vec<_>>();

            // NOTE: keep it here, the only use I see now, is for future logging/debugging
            // instances_total += instances_count;

            unsafe {
                indirect_buffer_offset += self.indirect_buffer.map_and_write_to_device_memory(
                    &self.gpu,
                    indirect_buffer_offset,
                    indirect_buffer_data.as_slice(),
                );
                instance_buffer_offset += self.instance_buffer.map_and_write_to_device_memory(
                    &self.gpu,
                    instance_buffer_offset,
                    instance_buffer_data.as_slice(),
                );
            }
            draw_count
        } else {
            0
        };

        DrawCount {
            only_mesh: only_mesh_draws_count as u32,
            only_mesh_indexed: only_mesh_indexed_draws_count as u32,
            skin_mesh: skin_mesh_draws_count as u32,
            skin_mesh_indexed: skin_mesh_indexed_draws_count as u32,
        }
    }

    fn register_mesh(&mut self, mesh_id: Id<Mesh>, assets: &Assets) -> Option<MeshLayout> {
        // check if the mesh is already in buffer
        if let Some(mesh_layout) = self.mesh_registry.get(&mesh_id) {
            return Some(*mesh_layout);
        }
        // try to get mesh to store it in buffer
        if let Some(mesh) = assets.get(mesh_id) {
            let vertex_data_and_skin_info = mesh
                .buffer::<VertexBufferSkinMeshLayout>()
                .map(|vertex_data| {
                    (
                        vertex_data,
                        VertexBufferSkinMeshLayout::vertex_size() as u64,
                        true,
                    )
                })
                .or_else(|| {
                    mesh.buffer::<VertexBufferOnlyMeshLayout>()
                        .map(|vertex_data| {
                            (
                                vertex_data,
                                VertexBufferOnlyMeshLayout::vertex_size() as u64,
                                false,
                            )
                        })
                });

            if let Some((vertex_data, vertex_size, has_skin)) = vertex_data_and_skin_info {
                let vertex_offset = if has_skin {
                    self.vertex_buffer_skin_mesh_usage
                } else {
                    self.vertex_buffer_only_mesh_usage
                };
                let index_data = mesh.indices::<u32>();

                let mesh_layout = MeshLayout {
                    vertices: LayoutInBuffer {
                        offset: vertex_offset,
                        base: (vertex_offset / vertex_size) as u32,
                        count: mesh.count_vertices() as u32,
                    },
                    indices: index_data.map(|data| LayoutInBuffer {
                        offset: self.index_buffer_usage,
                        base: (self.index_buffer_usage / (std::mem::size_of::<u32>() as u64))
                            as u32,
                        count: data.len() as u32,
                    }),
                    has_skin,
                };

                log::debug!(
                    "VB offset: {}, IB offset: {:?}, has skin: {:?}",
                    mesh_layout.vertices.offset,
                    mesh_layout.indices.as_ref().map(|i| i.offset),
                    has_skin
                );

                self.mesh_registry.insert(mesh_id, mesh_layout);

                unsafe {
                    if has_skin {
                        log::debug!(
                            "write {:?} bytes to VB(skin) at offset {:?}",
                            vertex_data.len(),
                            vertex_offset
                        );
                        self.vertex_buffer_skin_mesh_usage +=
                            self.vertex_buffer_skin_mesh.map_and_write_to_device_memory(
                                &self.gpu,
                                vertex_offset,
                                vertex_data.as_slice(),
                            );
                    } else {
                        log::debug!(
                            "write {:?} bytes to VB(no skin) at offset {:?}",
                            vertex_data.len(),
                            vertex_offset
                        );
                        self.vertex_buffer_only_mesh_usage +=
                            self.vertex_buffer_only_mesh.map_and_write_to_device_memory(
                                &self.gpu,
                                vertex_offset,
                                vertex_data.as_slice(),
                            );
                    }
                };

                // log::debug!("Indices @{}: {:?}", self.index_buffer_usage, index_data);
                if let Some(data) = index_data.as_ref() {
                    log::debug!(
                        "write {:?} bytes to IB at offset {:?}",
                        data.len() * std::mem::size_of::<u32>(),
                        self.index_buffer_usage
                    );
                    unsafe {
                        self.index_buffer_usage +=
                            self.index_buffer.map_and_write_to_device_memory(
                                &self.gpu,
                                self.index_buffer_usage,
                                data,
                            );
                    }
                    log::debug!("self.index_buffer_usage -> {}", self.index_buffer_usage);
                }

                return Some(mesh_layout);
            } else {
                log::error!("Could not store the mesh named `{}`", mesh.name());
            }
        }
        None
    }

    fn register_material(&mut self, material_id: Id<Material>, assets: &Assets) -> Option<u32> {
        let material_uniform: MaterialUniform = if let Some(material) = assets.get(material_id) {
            let mut staging_layer_count: u32 = 0;
            let (albedo_map_index, base_array_layer) = assets
                .get(material.albedo_map)
                .map(|image| {
                    let base_array_layer = self.material_layer_index.len();
                    if !self.material_layer_index.contains_key(&material.albedo_map) {
                        // write to buffer
                        // TODO: verify material extent
                        unsafe {
                            self.material_staging_buffer.map_and_write_to_device_memory(
                                &self.gpu,
                                (staging_layer_count
                                    * self.material_layer_size.width
                                    * self.material_layer_size.height
                                    * std::mem::size_of::<u32>() as u32)
                                    as u64,
                                image.data(),
                            );
                        };
                        self.material_layer_index
                            .insert(material.albedo_map, base_array_layer);
                        staging_layer_count += 1;
                    }
                    let albedo_map_index = self
                        .material_layer_index
                        .get(&material.albedo_map)
                        .cloned()
                        .expect("Layer index must be inserted at this stage")
                        as u32;
                    (albedo_map_index, base_array_layer)
                })
                .unwrap_or((0, 0));
            // flush material staging buffer
            unsafe {
                self.flush_material_staging_buffer(staging_layer_count, base_array_layer as u32);
            };
            let mut material_uniform: MaterialUniform = material.into();
            material_uniform.maps_1 = [
                albedo_map_index,
                std::u32::MAX,
                std::u32::MAX,
                std::u32::MAX,
            ];
            material_uniform.maps_2 = [std::u32::MAX; 4];
            material_uniform
        } else {
            return None;
        };
        // NOTE: most likely there will be need of having an index for relations
        // between Id<Image> and material layer index
        self.material_buffer_index
            .get(&material_id)
            .cloned()
            .map(|material_index| {
                self.material_buffer_data[material_index as usize] = material_uniform;
                material_index
            })
            .or_else(|| {
                let material_index = self.material_buffer_index.len() as u32;
                self.material_buffer_data.push(material_uniform);
                self.material_buffer_index
                    .insert(material_id, material_index);
                Some(material_index)
            })
    }

    unsafe fn flush_material_staging_buffer(
        &self,
        staging_layer_count: u32,
        base_array_layer: u32,
    ) {
        if staging_layer_count == 0 {
            return;
        }
        log::debug!("copy material to vk:Image ({} layers)", staging_layer_count);
        // begin
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

        // Command buffer
        // let buffer_copy_regions = (0..staging_layer_count)
        //    .into_iter()
        //    .map(|_| {
        //        vk::BufferImageCopy::default()
        //            .image_subresource(
        //                vk::ImageSubresourceLayers::default()
        //                    .aspect_mask(vk::ImageAspectFlags::COLOR)
        //                    .base_array_layer(base_array_layer)
        //                    .layer_count(1),
        //            )
        //            .image_extent(vk::Extent3D {
        //                width: self.material_layer_size.width,
        //                height: self.material_layer_size.height,
        //                depth: 1,
        //            })
        //    })
        //    .collect::<Vec<_>>();
        let buffer_copy_regions = [vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(base_array_layer)
                    .layer_count(staging_layer_count),
            )
            .image_extent(vk::Extent3D {
                width: self.material_layer_size.width,
                height: self.material_layer_size.height,
                depth: 1,
            })];
        let texture_barrier = vk::ImageMemoryBarrier {
            dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            image: self.material_image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: self.material_layer_count, // TODO: shouldn't there be total number?
                ..Default::default()
            },
            ..Default::default()
        };
        self.gpu.cmd_pipeline_barrier(
            self.command_buffer_setup,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[texture_barrier],
        );
        self.gpu.cmd_copy_buffer_to_image(
            self.command_buffer_setup,
            self.material_staging_buffer.handle,
            self.material_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            buffer_copy_regions.as_slice(),
        );
        let texture_barrier_end = vk::ImageMemoryBarrier {
            src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            dst_access_mask: vk::AccessFlags::SHADER_READ,
            old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, // TODO: shouldn't there be total number?
            image: self.material_image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: self.material_layer_count,
                ..Default::default()
            },
            ..Default::default()
        };
        self.gpu.cmd_pipeline_barrier(
            self.command_buffer_setup,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[texture_barrier_end],
        );
        // end
        self.gpu
            .end_command_buffer(self.command_buffer_setup)
            .expect("End commandbuffer");

        let command_buffers = vec![self.command_buffer_setup];
        let wait_semaphores = [];
        let wait_mask = [];
        let signal_semaphores = [];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        log::debug!("submit image copy");
        self.gpu
            .submit_queue(&[submit_info], self.command_buffer_setup_reuse_fence)
            .expect("queue submit failed.");
        // wait
        let fences = [self.command_buffer_setup_reuse_fence];
        self.gpu
            .wait_for_fences(&fences, true, std::u64::MAX)
            .expect("Failed to wait until end of textures bufering");
        log::debug!("wait_for_fences: done");
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

    unsafe fn create_transfer_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
        let buffer_create_info = vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
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

    unsafe fn create_graphics_pipelines(&self, surface_resolution: Extent2D) -> Vec<vk::Pipeline> {
        let shader_entry_point = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        // ONLY MESH
        let only_mesh_shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                module: self.shader_vertex_only_mesh,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: self.shader_fragment_only_mesh,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        // ONLY MESH: vertex binding
        let only_mesh_vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<VertexBufferOnlyMeshLayout>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let only_mesh_vertex_input_attribute_descriptions = [
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

        let only_mesh_vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&only_mesh_vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&only_mesh_vertex_input_binding_descriptions);
        let only_mesh_vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        // SKIN MESH
        let skin_mesh_shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                module: self.shader_vertex_skin_mesh,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: self.shader_fragment_skin_mesh,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];
        // SKIN MESH: vertex binding
        let skin_mesh_vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<VertexBufferSkinMeshLayout>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let skin_mesh_vertex_input_attribute_descriptions = [
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
            // weights
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: std::mem::size_of::<VertexPosition>() as u32
                    + std::mem::size_of::<VertexNormal>() as u32
                    + std::mem::size_of::<VertexTexture>() as u32,
            },
            // joints
            vk::VertexInputAttributeDescription {
                location: 4,
                binding: 0,
                format: vk::Format::R16G16B16A16_UINT,
                offset: std::mem::size_of::<VertexPosition>() as u32
                    + std::mem::size_of::<VertexNormal>() as u32
                    + std::mem::size_of::<VertexTexture>() as u32
                    + std::mem::size_of::<VertexWeights>() as u32,
            },
        ];

        let skin_mesh_vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&skin_mesh_vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&skin_mesh_vertex_input_binding_descriptions);
        let skin_mesh_vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
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

        let only_mesh_graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&only_mesh_shader_stages)
            .vertex_input_state(&only_mesh_vertex_input_state_info)
            .input_assembly_state(&only_mesh_vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(self.pipeline_layout_render)
            .render_pass(self.render_pass);

        let skin_mesh_graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&skin_mesh_shader_stages)
            .vertex_input_state(&skin_mesh_vertex_input_state_info)
            .input_assembly_state(&skin_mesh_vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(self.pipeline_layout_render)
            .render_pass(self.render_pass);

        self.gpu
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[
                    only_mesh_graphic_pipeline_info,
                    skin_mesh_graphic_pipeline_info,
                ],
            )
            .expect("Failed to create graphics pipelines")
    }

    unsafe fn destroy_graphics_pipelines(&self) {
        self.gpu.destroy_pipeline(self.pipeline_render_only_mesh);
        self.gpu.destroy_pipeline(self.pipeline_render_skin_mesh);
    }

    unsafe fn execute_render_pass(&self, frame: &Frame, draw_count: DrawCount) {
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
            self.pipeline_layout_render,
            0,
            &self.descriptor_sets[..],
            &[],
        );

        let mut offset: u64 = 0;

        // ONLY MESH
        if draw_count.only_mesh != 0 || draw_count.only_mesh_indexed != 0 {
            self.gpu.cmd_bind_pipeline(
                self.command_buffer_draw,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_render_only_mesh,
            );
            self.gpu
                .cmd_set_viewport(self.command_buffer_draw, 0, &viewports);
            self.gpu
                .cmd_set_scissor(self.command_buffer_draw, 0, &scissors);
            self.gpu.cmd_bind_vertex_buffers(
                self.command_buffer_draw,
                0,
                &[self.vertex_buffer_only_mesh.handle],
                &[0],
            );

            if draw_count.only_mesh != 0 {
                log::debug!(
                    "cmd_draw_indirect(offset: {}, draw_count: {}, stride: {})",
                    offset,
                    draw_count.only_mesh,
                    std::mem::size_of::<vk::DrawIndirectCommand>() as u32
                );
                self.gpu.cmd_draw_indirect(
                    self.command_buffer_draw,
                    self.indirect_buffer.handle,
                    offset,
                    draw_count.only_mesh,
                    std::mem::size_of::<vk::DrawIndirectCommand>() as u32,
                );
                offset += draw_count.only_mesh as u64
                    * std::mem::size_of::<vk::DrawIndirectCommand>() as u64;
            }

            if draw_count.only_mesh_indexed != 0 {
                // padding
                offset += offset % std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64;
                log::debug!(
                    "cmd_draw_indexed_indirect(offset: {}, draw_count: {}, stride: {})",
                    offset,
                    draw_count.only_mesh_indexed,
                    std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u32
                );
                self.gpu.cmd_bind_index_buffer(
                    self.command_buffer_draw,
                    self.index_buffer.handle,
                    0,
                    vk::IndexType::UINT32,
                );

                self.gpu.cmd_draw_indexed_indirect(
                    self.command_buffer_draw,
                    self.indirect_buffer.handle,
                    offset,
                    draw_count.only_mesh_indexed,
                    std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u32,
                );

                offset += draw_count.only_mesh_indexed as u64
                    * std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64;
            }
        }

        // MESH WITH SKIN
        if draw_count.skin_mesh != 0 || draw_count.skin_mesh_indexed != 0 {
            self.gpu.cmd_bind_pipeline(
                self.command_buffer_draw,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_render_skin_mesh,
            );
            self.gpu
                .cmd_set_viewport(self.command_buffer_draw, 0, &viewports);
            self.gpu
                .cmd_set_scissor(self.command_buffer_draw, 0, &scissors);
            self.gpu.cmd_bind_vertex_buffers(
                self.command_buffer_draw,
                0,
                &[self.vertex_buffer_skin_mesh.handle],
                &[0],
            );

            if draw_count.skin_mesh != 0 {
                // padding
                offset += offset % std::mem::size_of::<vk::DrawIndirectCommand>() as u64;
                log::debug!(
                    "skin - cmd_draw_indirect(offset: {}, draw_count: {}, stride: {})",
                    offset,
                    draw_count.skin_mesh,
                    std::mem::size_of::<vk::DrawIndirectCommand>() as u32
                );
                self.gpu.cmd_draw_indirect(
                    self.command_buffer_draw,
                    self.indirect_buffer.handle,
                    offset,
                    draw_count.skin_mesh,
                    std::mem::size_of::<vk::DrawIndirectCommand>() as u32,
                );
                offset += draw_count.skin_mesh as u64
                    * std::mem::size_of::<vk::DrawIndirectCommand>() as u64;
            }

            if draw_count.skin_mesh_indexed != 0 {
                offset += offset % std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64;
                log::debug!(
                    "skin - cmd_draw_indexed_indirect(offset: {}, draw_count: {}, stride: {})",
                    offset,
                    draw_count.skin_mesh_indexed,
                    std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u32
                );
                self.gpu.cmd_bind_index_buffer(
                    self.command_buffer_draw,
                    self.index_buffer.handle,
                    0,
                    vk::IndexType::UINT32,
                );

                self.gpu.cmd_draw_indexed_indirect(
                    self.command_buffer_draw,
                    self.indirect_buffer.handle,
                    offset,
                    draw_count.skin_mesh_indexed,
                    std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u32,
                );
            }
        }

        self.gpu.cmd_end_render_pass(self.command_buffer_draw);

        self.gpu
            .end_command_buffer(self.command_buffer_draw)
            .expect("End commandbuffer");

        // panic!("--------------------------- BREAKPOINT ---------------------------");
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

        let command_buffers = [self.command_buffer_setup];
        let wait_mask = [];
        let wait_semaphores = [];
        let signal_semaphores = [];

        let submits = [vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)];

        self.gpu
            .submit_queue(&submits, self.command_buffer_setup_reuse_fence)
            .expect("queue submit failed.")
    }

    unsafe fn submit_draw_commands(&self) {
        // panic!("----------------------- BREAKPOINT -----------------------");
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
    vertex_buffer_only_mesh_size: u64,
    vertex_buffer_skin_mesh_size: u64,
    indirect_buffer_size: u64,
    instance_buffer_size: u64,
    material_buffer_size: u64,
    transform_buffer_size: u64,
    material_count: u32,
    material_image_size: Extent2D,
}

impl Default for RenderModelsSetup {
    fn default() -> Self {
        Self {
            wait_semaphores: Vec::new(),
            surface_format: vk::Format::default(),
            index_buffer_size: 8 * 1024 * 1024,
            vertex_buffer_only_mesh_size: 8 * 1024 * 1024,
            vertex_buffer_skin_mesh_size: 8 * 1024 * 1024,
            indirect_buffer_size: 1000
                * std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64,
            instance_buffer_size: 1000 * std::mem::size_of::<InstanceUniform>() as u64,
            material_buffer_size: 1000 * std::mem::size_of::<MaterialUniform>() as u64,
            transform_buffer_size: 1000 * std::mem::size_of::<TransformUniform>() as u64,
            material_count: 2,
            material_image_size: Extent2D {
                width: 1024,
                height: 1024,
            },
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
    /// material index in buffer
    pub material_index: u32,
    /// transform index in buffer
    pub transform_index: u32,
    /// joint index in buffer
    pub joint_index: u32,
    /// padding
    pub _padding: u32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy, Default)]
pub struct TransformUniform {
    /// Model/Joint transform matrix
    pub transform: [[f32; 4]; 4],
}
