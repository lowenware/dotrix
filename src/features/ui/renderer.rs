use std::collections::HashMap;
use std::io::Cursor;

use ash::vk::Handle;
use bytemuck::{Pod, Zeroable};

use crate::graphics::{vk, Buffer, CommandRecorder, Gpu, RenderSubmit};
use crate::loaders::Assets;
use crate::models::Color;
use crate::utils::Id;
use crate::{Any, Display, Extent2D, Frame, Ref, Task};

use super::builder::{Overlay, UiDrawCommand};
use super::font::Font;

/// Maximum UI instances (rects + glyphs) per frame.
pub const UI_MAX_INSTANCES: usize = 8192;

/// Maximum font atlas layers in the texture array.
pub const UI_MAX_FONT_LAYERS: u32 = 16;

/// GPU font texture dimensions (atlas pixels are uploaded to the top-left corner).
pub const UI_FONT_TEXTURE_SIZE: u32 = 512;

const UI_KIND_RECT: f32 = 0.0;
const UI_KIND_GLYPH: f32 = 1.0;

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
struct UiGlobals {
    resolution: [f32; 2],
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct UiInstance {
    pub rect: [f32; 4],
    pub color: [f32; 4],
    pub params: [f32; 4],
    pub uv: [f32; 4],
}

const QUAD_VERTICES: [[f32; 2]; 6] = [
    [0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
    [0.0, 0.0],
    [1.0, 1.0],
    [0.0, 1.0],
];

pub struct RenderOverlay {
    gpu: Gpu,
    surface_version: u64,
    wait_for: Vec<std::any::TypeId>,
    command_pool: vk::CommandPool,
    command_buffer_setup: vk::CommandBuffer,
    command_buffer_setup_reuse_fence: vk::Fence,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
    indirect_buffer: Buffer,
    globals_buffer: Buffer,
    font_image: vk::Image,
    font_image_memory: vk::DeviceMemory,
    font_image_view: vk::ImageView,
    font_sampler: vk::Sampler,
    font_staging_buffer: Buffer,
    font_layer_index: HashMap<Id<Font>, u32>,
    font_layer_count: u32,
    descriptor_pool: vk::DescriptorPool,
    desc_set_layouts: [vk::DescriptorSetLayout; 1],
    descriptor_sets: Vec<vk::DescriptorSet>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    shader_vertex: vk::ShaderModule,
    shader_fragment: vk::ShaderModule,
    instance_count: u32,
}

impl Drop for RenderOverlay {
    fn drop(&mut self) {
        unsafe {
            self.gpu.device_wait_idle().unwrap();
            if !self.pipeline.is_null() {
                self.gpu.destroy_pipeline(self.pipeline);
            }
            self.gpu.destroy_pipeline_layout(self.pipeline_layout);
            self.gpu.destroy_shader_module(self.shader_vertex);
            self.gpu.destroy_shader_module(self.shader_fragment);
            self.vertex_buffer.free_memory_and_destroy(&self.gpu);
            self.instance_buffer.free_memory_and_destroy(&self.gpu);
            self.indirect_buffer.free_memory_and_destroy(&self.gpu);
            self.globals_buffer.free_memory_and_destroy(&self.gpu);
            self.font_staging_buffer.free_memory_and_destroy(&self.gpu);
            self.gpu.destroy_sampler(self.font_sampler);
            self.gpu.destroy_image_view(self.font_image_view);
            self.gpu.free_memory(self.font_image_memory);
            self.gpu.destroy_image(self.font_image);
            for layout in &self.desc_set_layouts {
                self.gpu.destroy_descriptor_set_layout(*layout);
            }
            self.gpu.destroy_descriptor_pool(self.descriptor_pool);
            self.gpu.destroy_command_pool(self.command_pool);
            self.gpu
                .destroy_fence(self.command_buffer_setup_reuse_fence);
        }
    }
}

impl Task for RenderOverlay {
    type Context = (Any<Overlay>, Any<Frame>, Ref<Display>, Ref<Assets>);
    type Output = RenderSubmit;

    fn run(&mut self, (overlay, frame, display, assets): Self::Context) -> Self::Output {
        if let Some(surface_version) = display.surface_changed(self.surface_version) {
            unsafe {
                self.gpu.device_wait_idle().unwrap();
                if self.pipeline.is_null() {
                    self.pipeline = self.create_graphics_pipeline(
                        display.render_pass(),
                        display.surface_resolution(),
                    );
                }
            }
            self.surface_version = surface_version;
        }

        if overlay.is_empty() {
            return RenderSubmit::skip::<Self>(self.wait_for.as_slice());
        }

        self.instance_count = self.build_instances(&overlay, &assets);

        if self.instance_count == 0 {
            return RenderSubmit::skip::<Self>(self.wait_for.as_slice());
        }

        let globals = [UiGlobals {
            resolution: [
                frame.resolution.width as f32,
                frame.resolution.height as f32,
            ],
            _padding: [0.0, 0.0],
        }];
        unsafe {
            self.globals_buffer
                .map_and_write_to_device_memory(&self.gpu, 0, &globals);
        }

        let indirect = [vk::DrawIndirectCommand {
            vertex_count: 6,
            instance_count: self.instance_count,
            first_vertex: 0,
            first_instance: 0,
        }];
        unsafe {
            self.indirect_buffer
                .map_and_write_to_device_memory(&self.gpu, 0, &indirect);
        }

        let recorder = Recorder {
            resolution: frame.resolution,
            pipeline_layout: self.pipeline_layout,
            pipeline: self.pipeline,
            descriptor_sets: self.descriptor_sets.clone(),
            vertex_buffer: self.vertex_buffer.handle,
            indirect_buffer: self.indirect_buffer.handle,
        };

        RenderSubmit::new::<Self>(Box::new(recorder), &self.wait_for)
    }
}

impl RenderOverlay {
    pub fn setup() -> RenderOverlaySetup {
        RenderOverlaySetup::default()
    }

    pub fn new(display: &mut Display, setup: RenderOverlaySetup) -> Self {
        let gpu = display.gpu();

        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(gpu.queue_family_index());
        let command_pool = unsafe {
            gpu.create_command_pool(&pool_create_info)
                .expect("Failed to create UI command pool")
        };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);
        let command_buffer_setup = unsafe {
            gpu.allocate_command_buffers(&command_buffer_allocate_info)
                .into()
        };

        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let command_buffer_setup_reuse_fence = unsafe { gpu.create_fence(&fence_create_info) };

        let vertex_buffer = unsafe {
            create_vertex_buffer(
                &gpu,
                (QUAD_VERTICES.len() * std::mem::size_of::<[f32; 2]>()) as u64,
            )
            .expect("Could not allocate UI vertex buffer")
        };
        unsafe {
            vertex_buffer.map_and_write_to_device_memory(&gpu, 0, &QUAD_VERTICES);
        }

        let instance_buffer = unsafe {
            create_storage_buffer(
                &gpu,
                UI_MAX_INSTANCES as u64 * std::mem::size_of::<UiInstance>() as u64,
            )
            .expect("Could not allocate UI instance buffer")
        };

        let indirect_buffer = unsafe {
            create_indirect_buffer(&gpu, std::mem::size_of::<vk::DrawIndirectCommand>() as u64)
                .expect("Could not allocate UI indirect buffer")
        };

        let globals_buffer = unsafe {
            create_uniform_buffer(&gpu, std::mem::size_of::<UiGlobals>() as u64)
                .expect("Could not allocate UI globals buffer")
        };

        let font_layer_count = UI_MAX_FONT_LAYERS;
        let font_image_create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8_UNORM,
            extent: vk::Extent3D {
                width: UI_FONT_TEXTURE_SIZE,
                height: UI_FONT_TEXTURE_SIZE,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: font_layer_count,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let font_image = unsafe {
            gpu.create_image(&font_image_create_info)
                .expect("Failed to create UI font image")
        };
        let font_image_memory_req = unsafe { gpu.get_image_memory_requirements(font_image) };
        let font_image_memory_index = gpu
            .find_memory_type_index(
                &font_image_memory_req,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .expect("Unable to find memory for UI font image");
        let font_image_memory = unsafe {
            gpu.allocate_memory(&vk::MemoryAllocateInfo {
                allocation_size: font_image_memory_req.size,
                memory_type_index: font_image_memory_index,
                ..Default::default()
            })
            .expect("Failed to allocate UI font image memory")
        };
        unsafe {
            gpu.bind_image_memory(font_image, font_image_memory, 0)
                .expect("Failed to bind UI font image memory");
        }

        let font_image_view = unsafe {
            gpu.create_image_view(&vk::ImageViewCreateInfo {
                view_type: vk::ImageViewType::TYPE_2D_ARRAY,
                format: vk::Format::R8_UNORM,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: font_layer_count,
                    ..Default::default()
                },
                image: font_image,
                ..Default::default()
            })
            .expect("Failed to create UI font image view")
        };

        let font_sampler = unsafe {
            gpu.create_sampler(&vk::SamplerCreateInfo {
                mag_filter: vk::Filter::LINEAR,
                min_filter: vk::Filter::LINEAR,
                address_mode_u: vk::SamplerAddressMode::CLAMP_TO_EDGE,
                address_mode_v: vk::SamplerAddressMode::CLAMP_TO_EDGE,
                address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE,
                ..Default::default()
            })
            .expect("Failed to create UI font sampler")
        };

        let font_staging_buffer = unsafe {
            create_transfer_buffer(&gpu, (UI_FONT_TEXTURE_SIZE * UI_FONT_TEXTURE_SIZE) as u64)
                .expect("Could not allocate UI font staging buffer")
        };

        let descriptor_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
            },
        ];
        let descriptor_pool = unsafe {
            gpu.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .pool_sizes(&descriptor_sizes)
                    .max_sets(1),
            )
            .unwrap()
        };

        let desc_layout_bindings = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 2,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];
        let desc_set_layouts = unsafe {
            [gpu.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&desc_layout_bindings),
            )
            .unwrap()]
        };

        let descriptor_sets = unsafe {
            gpu.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&desc_set_layouts),
            )
            .unwrap()
        };

        let globals_descriptor = vk::DescriptorBufferInfo {
            buffer: globals_buffer.handle,
            offset: 0,
            range: globals_buffer.size,
        };
        let instance_descriptor = vk::DescriptorBufferInfo {
            buffer: instance_buffer.handle,
            offset: 0,
            range: instance_buffer.size,
        };
        let font_descriptor = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: font_image_view,
            sampler: font_sampler,
        };

        unsafe {
            gpu.update_descriptor_sets(
                &[
                    vk::WriteDescriptorSet {
                        dst_binding: 0,
                        dst_set: descriptor_sets[0],
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        p_buffer_info: &globals_descriptor,
                        ..Default::default()
                    },
                    vk::WriteDescriptorSet {
                        dst_binding: 1,
                        dst_set: descriptor_sets[0],
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                        p_buffer_info: &instance_descriptor,
                        ..Default::default()
                    },
                    vk::WriteDescriptorSet {
                        dst_binding: 2,
                        dst_set: descriptor_sets[0],
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        p_image_info: &font_descriptor,
                        ..Default::default()
                    },
                ],
                &[],
            );
        }

        let pipeline_layout = unsafe {
            gpu.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().set_layouts(&desc_set_layouts),
            )
            .expect("Failed to create UI pipeline layout")
        };

        let shader_vertex = unsafe {
            load_shader_module(&gpu, include_bytes!("shaders/ui.vert.spv"))
                .expect("Failed to load UI vertex shader")
        };
        let shader_fragment = unsafe {
            load_shader_module(&gpu, include_bytes!("shaders/ui.frag.spv"))
                .expect("Failed to load UI fragment shader")
        };

        Self {
            gpu,
            surface_version: 0,
            wait_for: setup.wait_for,
            command_pool,
            command_buffer_setup,
            command_buffer_setup_reuse_fence,
            vertex_buffer,
            instance_buffer,
            indirect_buffer,
            globals_buffer,
            font_image,
            font_image_memory,
            font_image_view,
            font_sampler,
            font_staging_buffer,
            font_layer_index: HashMap::new(),
            font_layer_count,
            descriptor_pool,
            desc_set_layouts,
            descriptor_sets,
            pipeline_layout,
            pipeline: vk::Pipeline::null(),
            shader_vertex,
            shader_fragment,
            instance_count: 0,
        }
    }

    fn build_instances(&mut self, overlay: &Overlay, assets: &Assets) -> u32 {
        let mut instances = Vec::with_capacity(overlay.commands.len());

        for command in &overlay.commands {
            if instances.len() >= UI_MAX_INSTANCES {
                crate::log::warn!(
                    "UI instance count exceeded UI_MAX_INSTANCES ({})",
                    UI_MAX_INSTANCES
                );
                break;
            }

            match command {
                UiDrawCommand::Rect {
                    rect,
                    color,
                    corner_radius,
                } => {
                    instances.push(UiInstance {
                        rect: [rect.x, rect.y, rect.width, rect.height],
                        color: color_to_array(*color),
                        params: [UI_KIND_RECT, *corner_radius, 0.0, 0.0],
                        uv: [0.0, 0.0, 1.0, 1.0],
                    });
                }
                UiDrawCommand::Glyph {
                    rect,
                    color,
                    font,
                    uv_min,
                    uv_max,
                } => {
                    let layer = self.register_font(*font, assets);
                    if layer.is_none() {
                        continue;
                    }
                    let font_asset = assets.get(*font).expect("font must exist");
                    // UVs are normalized to the CPU atlas size; scale to the fixed GPU texture.
                    let uv_scale_x = font_asset.atlas_width() as f32 / UI_FONT_TEXTURE_SIZE as f32;
                    let uv_scale_y = font_asset.atlas_height() as f32 / UI_FONT_TEXTURE_SIZE as f32;
                    instances.push(UiInstance {
                        rect: [rect.x, rect.y, rect.width, rect.height],
                        color: color_to_array(*color),
                        params: [UI_KIND_GLYPH, 0.0, layer.unwrap() as f32, 0.0],
                        uv: [
                            uv_min[0] * uv_scale_x,
                            uv_min[1] * uv_scale_y,
                            uv_max[0] * uv_scale_x,
                            uv_max[1] * uv_scale_y,
                        ],
                    });
                }
            }
        }

        if instances.is_empty() {
            return 0;
        }

        unsafe {
            self.instance_buffer
                .map_and_write_to_device_memory(&self.gpu, 0, instances.as_slice());
        }

        instances.len() as u32
    }

    fn register_font(&mut self, font_id: Id<Font>, assets: &Assets) -> Option<u32> {
        if let Some(layer) = self.font_layer_index.get(&font_id) {
            return Some(*layer);
        }
        if self.font_layer_index.len() as u32 >= self.font_layer_count {
            crate::log::error!("UI font layer limit ({}) exceeded", self.font_layer_count);
            return None;
        }

        let font = assets.get(font_id)?;
        let layer = self.font_layer_index.len() as u32;
        let width = font.atlas_width().max(1);
        let height = font.atlas_height().max(1);
        if width > UI_FONT_TEXTURE_SIZE || height > UI_FONT_TEXTURE_SIZE {
            crate::log::error!(
                "font atlas {}x{} exceeds UI_FONT_TEXTURE_SIZE ({}); glyphs will be clipped",
                width,
                height,
                UI_FONT_TEXTURE_SIZE
            );
        }
        let pixels = font.atlas_pixels();

        unsafe {
            self.font_staging_buffer
                .map_and_write_to_device_memory(&self.gpu, 0, pixels);
            self.upload_font_layer(layer, width, height);
        }

        self.font_layer_index.insert(font_id, layer);
        Some(layer)
    }

    unsafe fn upload_font_layer(&self, layer: u32, width: u32, height: u32) {
        self.gpu
            .wait_for_fences(&[self.command_buffer_setup_reuse_fence], true, u64::MAX)
            .expect("UI font upload fence wait failed");
        self.gpu
            .reset_fences(&[self.command_buffer_setup_reuse_fence])
            .expect("UI font upload fence reset failed");
        self.gpu
            .reset_command_buffer(
                self.command_buffer_setup,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("UI font upload command buffer reset failed");
        self.gpu
            .begin_command_buffer(
                self.command_buffer_setup,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )
            .expect("UI font upload command buffer begin failed");

        let barrier_to_transfer = vk::ImageMemoryBarrier {
            dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            image: self.font_image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: self.font_layer_count,
                ..Default::default()
            },
            ..Default::default()
        };
        self.gpu.cmd_pipeline_barrier(
            self.command_buffer_setup,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier_to_transfer],
        );

        self.gpu.cmd_copy_buffer_to_image(
            self.command_buffer_setup,
            self.font_staging_buffer.handle,
            self.font_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[vk::BufferImageCopy::default()
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(layer)
                        .layer_count(1),
                )
                .image_extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })],
        );

        let barrier_to_shader = vk::ImageMemoryBarrier {
            src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            dst_access_mask: vk::AccessFlags::SHADER_READ,
            old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image: self.font_image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: self.font_layer_count,
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
            &[barrier_to_shader],
        );

        self.gpu
            .end_command_buffer(self.command_buffer_setup)
            .expect("UI font upload command buffer end failed");

        self.gpu
            .submit_queue(
                &[vk::SubmitInfo::default().command_buffers(&[self.command_buffer_setup])],
                self.command_buffer_setup_reuse_fence,
            )
            .expect("UI font upload submit failed");
        self.gpu
            .wait_for_fences(&[self.command_buffer_setup_reuse_fence], true, u64::MAX)
            .expect("UI font upload completion wait failed");
    }

    unsafe fn create_graphics_pipeline(
        &self,
        render_pass: vk::RenderPass,
        surface_resolution: Extent2D,
    ) -> vk::Pipeline {
        let shader_entry_point = c"main";
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                module: self.shader_vertex,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                module: self.shader_fragment,
                p_name: shader_entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let vertex_bindings = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: 8,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let vertex_attributes = [vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 0,
        }];

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_bindings)
            .vertex_attribute_descriptions(&vertex_attributes);
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
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
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: surface_resolution.width,
                height: surface_resolution.height,
            },
        }];
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterization = vk::PipelineRasterizationStateCreateInfo {
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            ..Default::default()
        };
        let multisample = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 0,
            depth_write_enable: 0,
            ..Default::default()
        };
        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachments);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_state);

        self.gpu
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[vk::GraphicsPipelineCreateInfo::default()
                    .stages(&shader_stages)
                    .vertex_input_state(&vertex_input)
                    .input_assembly_state(&input_assembly)
                    .viewport_state(&viewport_state)
                    .rasterization_state(&rasterization)
                    .multisample_state(&multisample)
                    .depth_stencil_state(&depth_stencil)
                    .color_blend_state(&color_blend)
                    .dynamic_state(&dynamic_state_info)
                    .layout(self.pipeline_layout)
                    .render_pass(render_pass)],
            )
            .expect("Failed to create UI graphics pipeline")[0]
    }
}

fn color_to_array(color: Color<f32>) -> [f32; 4] {
    [color.r, color.g, color.b, color.a]
}

pub struct RenderOverlaySetup {
    wait_for: Vec<std::any::TypeId>,
}

impl Default for RenderOverlaySetup {
    fn default() -> Self {
        Self {
            wait_for: Vec::new(),
        }
    }
}

impl RenderOverlaySetup {
    pub fn wait_for(mut self, dependencies: impl IntoIterator<Item = std::any::TypeId>) -> Self {
        self.wait_for.extend(dependencies);
        self
    }

    pub fn create(self, display: &mut Display) -> RenderOverlay {
        RenderOverlay::new(display, self)
    }
}

struct Recorder {
    resolution: Extent2D,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_sets: Vec<vk::DescriptorSet>,
    vertex_buffer: vk::Buffer,
    indirect_buffer: vk::Buffer,
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
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: self.resolution.width,
                height: self.resolution.height,
            },
        }];

        gpu.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline,
        );
        gpu.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            &self.descriptor_sets,
            &[],
        );
        gpu.cmd_set_viewport(command_buffer, 0, &viewports);
        gpu.cmd_set_scissor(command_buffer, 0, &scissors);
        gpu.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer], &[0]);
        gpu.cmd_draw_indirect(
            command_buffer,
            self.indirect_buffer,
            0,
            1,
            std::mem::size_of::<vk::DrawIndirectCommand>() as u32,
        );
    }
}

unsafe fn create_vertex_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
    Buffer::create_and_allocate(
        gpu,
        &vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
    )
}

unsafe fn create_storage_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
    Buffer::create_and_allocate(
        gpu,
        &vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::STORAGE_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
    )
}

unsafe fn create_indirect_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
    Buffer::create_and_allocate(
        gpu,
        &vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::INDIRECT_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
    )
}

unsafe fn create_uniform_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
    Buffer::create_and_allocate(
        gpu,
        &vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
    )
}

unsafe fn create_transfer_buffer(gpu: &Gpu, size: u64) -> Result<Buffer, vk::Result> {
    Buffer::create_and_allocate(
        gpu,
        &vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
    )
}

unsafe fn load_shader_module(gpu: &Gpu, bytes: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
    let mut cursor = Cursor::new(bytes);
    let shader_code = ash::util::read_spv(&mut cursor).expect("Failed to read UI shader SPV");
    gpu.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&shader_code))
}
