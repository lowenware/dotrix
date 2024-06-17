use std::borrow::Cow;
use std::ffi::{c_char, CStr, CString};
use std::sync::Arc;

pub use ash::vk;
use ash::vk::DescriptorPool;

use crate::log;
use crate::window;

use super::{DeviceType, DisplaySetup, Extent2D};

/*
 * TODO:
 * - get rid of constructors, use methods on GPU instead to create most of data types
 * - reuse Vulkan *CreateInfo data struct provided by ash
 */

// TODO: consider making this private
struct Device {
    queue_family_index: u32,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    vk_queue: vk::Queue,
    vk_device: ash::Device,
    vk_instance: ash::Instance,
    _vk_entry: ash::Entry,

    vk_debug: Option<(ash::ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT)>,
}

impl Drop for Device {
    fn drop(&mut self) {
        log::debug!("Device::drop");
        unsafe {
            self.vk_device.device_wait_idle().unwrap();

            if let Some((vk_debug_utils, vk_debug_callback)) = self.vk_debug.take() {
                vk_debug_utils.destroy_debug_utils_messenger(vk_debug_callback, None);
            }

            self.vk_device.destroy_device(None);
            self.vk_instance.destroy_instance(None);
        }
    }
}

pub struct Surface {
    loader: ash::khr::surface::Instance,
    vk_surface: vk::SurfaceKHR,
    vk_surface_format: vk::SurfaceFormatKHR,
    vk_surface_capabilities: vk::SurfaceCapabilitiesKHR,
    vk_surface_transform: vk::SurfaceTransformFlagsKHR,
    vk_present_mode: vk::PresentModeKHR,
    vk_surface_resolution: vk::Extent2D,
    images_count: u32,
    version: u64,
}

impl Surface {
    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn resolution(&self) -> Extent2D {
        Extent2D {
            width: self.vk_surface_resolution.width,
            height: self.vk_surface_resolution.height,
        }
    }

    unsafe fn new(
        window: &window::Instance,
        vk_instance: &ash::Instance,
        vk_entry: &ash::Entry,
    ) -> Self {
        let raw_display_handle = window
            .display_handle()
            .expect("Can't get display handle")
            .as_raw();
        let raw_window_handle = window
            .window_handle()
            .expect("Can't get window handle")
            .as_raw();
        let window_resolution = window.resolution();
        let vk_surface_resolution = vk::Extent2D {
            width: window_resolution.width,
            height: window_resolution.height,
        };

        let vk_surface = ash_window::create_surface(
            vk_entry,
            vk_instance,
            raw_display_handle,
            raw_window_handle,
            None,
        )
        .expect("Failed to create a Vulkan surface");

        let loader = ash::khr::surface::Instance::new(vk_entry, vk_instance);

        Self {
            loader,
            vk_surface,
            vk_surface_format: vk::SurfaceFormatKHR::default(),
            vk_surface_capabilities: vk::SurfaceCapabilitiesKHR::default(),
            vk_surface_transform: vk::SurfaceTransformFlagsKHR::default(),
            vk_present_mode: vk::PresentModeKHR::default(),
            vk_surface_resolution,
            images_count: 2,
            version: 1,
        }
    }

    fn configure(&mut self, p_device: vk::PhysicalDevice) {
        let vk_surface_format = unsafe {
            self.loader
                .get_physical_device_surface_formats(p_device, self.vk_surface)
                .unwrap()[0]
        };

        let vk_surface_capabilities = unsafe {
            self.loader
                .get_physical_device_surface_capabilities(p_device, self.vk_surface)
                .unwrap()
        };

        let images_count = if vk_surface_capabilities.max_image_count > 0 {
            (vk_surface_capabilities.min_image_count + 1)
                .min(vk_surface_capabilities.max_image_count)
        } else {
            vk_surface_capabilities.min_image_count + 1
        };

        let vk_surface_resolution = match vk_surface_capabilities.current_extent.width {
            u32::MAX => vk::Extent2D {
                // NOTE: vk_surface_resolution comes from window size
                width: self.vk_surface_resolution.width,
                height: self.vk_surface_resolution.height,
            },
            _ => vk_surface_capabilities.current_extent,
        };

        let vk_surface_transform = if vk_surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            vk_surface_capabilities.current_transform
        };

        let present_modes = unsafe {
            self.loader
                .get_physical_device_surface_present_modes(p_device, self.vk_surface)
                .unwrap()
        };

        let vk_present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        self.vk_surface_format = vk_surface_format;
        self.vk_surface_capabilities = vk_surface_capabilities;
        self.vk_surface_transform = vk_surface_transform;
        self.vk_present_mode = vk_present_mode;
        self.images_count = images_count;
        self.vk_surface_resolution = vk_surface_resolution;
        self.images_count = images_count;
    }

    unsafe fn get_physical_device_support(
        &self,
        p_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> bool {
        self.loader
            .get_physical_device_surface_support(p_device, queue_family_index, self.vk_surface)
            .unwrap_or(false)
    }
}

struct Swapchain {
    loader: ash::khr::swapchain::Device,
    vk_swapchain: vk::SwapchainKHR,
    _vk_present_images: Vec<vk::Image>,
    vk_present_image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    unsafe fn create(gpu: &Gpu, surface: &Surface) -> Swapchain {
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.vk_surface)
            .min_image_count(surface.images_count)
            .image_color_space(surface.vk_surface_format.color_space)
            .image_format(surface.vk_surface_format.format)
            .image_extent(surface.vk_surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface.vk_surface_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(surface.vk_present_mode)
            .clipped(true)
            .image_array_layers(1);

        let loader =
            ash::khr::swapchain::Device::new(&gpu.device.vk_instance, &gpu.device.vk_device);

        let vk_swapchain = loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create a Vulkan swapchain");

        let vk_present_images = unsafe { loader.get_swapchain_images(vk_swapchain).unwrap() };

        let vk_present_image_views: Vec<vk::ImageView> = vk_present_images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::default()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface.vk_surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);

                unsafe { gpu.create_image_view(&create_view_info).unwrap() }
            })
            .collect();

        Swapchain {
            loader,
            vk_swapchain,
            _vk_present_images: vk_present_images,
            vk_present_image_views,
        }
    }

    unsafe fn destroy(&self, gpu: &Gpu) {
        for &image_view in self.vk_present_image_views.iter() {
            gpu.destroy_image_view(image_view);
        }

        self.loader.destroy_swapchain(self.vk_swapchain, None);
    }
}

/*
pub struct Framebuffers {
    vk_framebuffers: Vec<vk::Framebuffer>,
}

impl Framebuffers {
    pub fn new() -> Self {
        Self {
            vk_framebuffers: vec![],
        }
    }

    pub unsafe fn rebuild(&mut self, display: &Display, render_pass: vk::RenderPass) {
        self.destroy(&display.gpu);
        self.create(display, render_pass);
    }

    pub unsafe fn create(&mut self, display: &Display, render_pass: vk::RenderPass) {
        let resolution = display.surface_resolution();
        self.vk_framebuffers = display
            .swapchain
            .vk_present_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view /*, base.depth_image_view*/];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&framebuffer_attachments)
                    .width(resolution.width)
                    .height(resolution.height)
                    .layers(1);

                display
                    .gpu
                    .create_framebuffer(&frame_buffer_create_info)
                    .expect("Could not create a framebuffer")
            })
            .collect::<Vec<_>>()
    }

    pub unsafe fn destroy<'a>(&'a self, gpu: &Gpu) {
        for framebuffer in self.vk_framebuffers.iter() {
            gpu.destroy_framebuffer(*framebuffer);
        }
    }

    pub unsafe fn get(&self, swapchain_index: u32) -> vk::Framebuffer {
        self.vk_framebuffers[swapchain_index as usize]
    }
}

 */

#[derive(Clone)]
pub struct Semaphore {
    inner: Arc<SemaphoreInner>,
}

impl Semaphore {
    pub fn vk_semaphore(&self) -> &vk::Semaphore {
        &self.inner.vk_semaphore
    }
}

struct SemaphoreInner {
    vk_semaphore: vk::Semaphore,
    device: Arc<Device>,
}

impl Drop for SemaphoreInner {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_device
                .destroy_semaphore(self.vk_semaphore, None);
        };
    }
}

/// GPU abstraction layer
#[derive(Clone)]
pub struct Gpu {
    device: Arc<Device>,
    // queue: Arc<Queue>,
}

impl Gpu {
    pub fn device_memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.device.memory_properties
    }

    // Vulkan API:

    #[inline(always)]
    pub unsafe fn device_wait_idle(&self) -> Result<(), vk::Result> {
        self.device.vk_device.device_wait_idle()
    }

    #[inline(always)]
    pub unsafe fn allocate_command_buffers(
        &self,
        command_buffer_allocate_info: &vk::CommandBufferAllocateInfo,
    ) -> CommandBufferIter {
        CommandBufferIter {
            inner: self
                .device
                .vk_device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffers")
                .into_iter(),
        }
    }

    #[inline(always)]
    pub unsafe fn allocate_memory(
        &self,
        memory_allocate_info: &vk::MemoryAllocateInfo,
    ) -> Result<vk::DeviceMemory, vk::Result> {
        self.device
            .vk_device
            .allocate_memory(memory_allocate_info, None)
    }

    #[inline(always)]
    pub unsafe fn free_memory(&self, device_memory: vk::DeviceMemory) {
        self.device.vk_device.free_memory(device_memory, None)
    }

    #[inline(always)]
    pub unsafe fn begin_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        command_buffer_begin_info: &vk::CommandBufferBeginInfo,
    ) -> Result<(), vk::Result> {
        self.device
            .vk_device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
    }

    pub unsafe fn reset_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        flags: vk::CommandBufferResetFlags,
    ) -> Result<(), vk::Result> {
        self.device
            .vk_device
            .reset_command_buffer(command_buffer, flags)
    }

    #[inline(always)]
    pub unsafe fn end_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), vk::Result> {
        self.device.vk_device.end_command_buffer(command_buffer)
    }

    #[inline(always)]
    pub unsafe fn create_image_view(
        &self,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<vk::ImageView, vk::Result> {
        self.device.vk_device.create_image_view(create_info, None)
    }

    #[inline(always)]
    pub unsafe fn destroy_image_view(&self, image_view: vk::ImageView) {
        self.device.vk_device.destroy_image_view(image_view, None)
    }

    #[inline(always)]
    pub unsafe fn create_image(
        &self,
        create_info: &vk::ImageCreateInfo,
    ) -> Result<vk::Image, vk::Result> {
        self.device.vk_device.create_image(create_info, None)
    }

    #[inline(always)]
    pub unsafe fn destroy_image(&self, image: vk::Image) {
        self.device.vk_device.destroy_image(image, None)
    }

    #[inline(always)]
    pub unsafe fn get_image_memory_requirements(&self, image: vk::Image) -> vk::MemoryRequirements {
        self.device.vk_device.get_image_memory_requirements(image)
    }

    #[inline(always)]
    pub unsafe fn bind_image_memory(
        &self,
        image: vk::Image,
        image_memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
    ) -> Result<(), vk::Result> {
        self.device
            .vk_device
            .bind_image_memory(image, image_memory, offset)
    }

    #[inline(always)]
    pub unsafe fn create_framebuffer(
        &self,
        create_info: &vk::FramebufferCreateInfo,
    ) -> Result<vk::Framebuffer, vk::Result> {
        self.device.vk_device.create_framebuffer(create_info, None)
    }

    #[inline(always)]
    pub unsafe fn destroy_framebuffer(&self, framebuffer: vk::Framebuffer) {
        self.device.vk_device.destroy_framebuffer(framebuffer, None)
    }

    #[inline(always)]
    pub unsafe fn create_semaphore(
        &self,
        semaphore_create_info: &vk::SemaphoreCreateInfo,
    ) -> Result<vk::Semaphore, vk::Result> {
        self.device
            .vk_device
            .create_semaphore(semaphore_create_info, None)
    }

    #[inline(always)]
    pub unsafe fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        self.device.vk_device.destroy_semaphore(semaphore, None)
    }

    #[inline(always)]
    pub unsafe fn create_command_pool(
        &self,
        pool_create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<vk::CommandPool, vk::Result> {
        self.device
            .vk_device
            .create_command_pool(&pool_create_info, None)
    }

    #[inline(always)]
    pub unsafe fn get_buffer_memory_requirements(
        &self,
        buffer: vk::Buffer,
    ) -> vk::MemoryRequirements {
        self.device.vk_device.get_buffer_memory_requirements(buffer)
    }

    #[inline(always)]
    pub unsafe fn create_buffer(
        &self,
        buffer_create_info: &vk::BufferCreateInfo,
    ) -> Result<vk::Buffer, vk::Result> {
        self.device
            .vk_device
            .create_buffer(buffer_create_info, None)
    }

    #[inline(always)]
    pub unsafe fn destroy_buffer(&self, buffer: vk::Buffer) {
        self.device.vk_device.destroy_buffer(buffer, None);
    }

    #[inline(always)]
    pub unsafe fn bind_buffer_memory(
        &self,
        buffer: vk::Buffer,
        device_memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
    ) -> Result<(), vk::Result> {
        self.device
            .vk_device
            .bind_buffer_memory(buffer, device_memory, offset)
    }

    #[inline(always)]
    pub unsafe fn map_memory(
        &self,
        device_memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
        memory_map_flags: vk::MemoryMapFlags,
    ) -> Result<*mut std::ffi::c_void, vk::Result> {
        self.device
            .vk_device
            .map_memory(device_memory, offset, size, memory_map_flags)
    }

    #[inline(always)]
    pub unsafe fn unmap_memory(&self, device_memory: vk::DeviceMemory) {
        self.device.vk_device.unmap_memory(device_memory)
    }

    #[inline(always)]
    pub unsafe fn create_render_pass(
        &self,
        render_pass_create_info: &vk::RenderPassCreateInfo,
    ) -> Result<vk::RenderPass, vk::Result> {
        self.device
            .vk_device
            .create_render_pass(render_pass_create_info, None)
    }

    #[inline(always)]
    pub unsafe fn destroy_command_pool(&self, command_pool: vk::CommandPool) {
        self.device
            .vk_device
            .destroy_command_pool(command_pool, None);
    }

    #[inline(always)]
    pub unsafe fn destroy_render_pass(&self, render_pass: vk::RenderPass) {
        self.device.vk_device.destroy_render_pass(render_pass, None);
    }

    #[inline(always)]
    pub unsafe fn submit_queue(
        &self,
        submits: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> Result<(), vk::Result> {
        // NOTE: this function has mirrored name due to miss match in parameters with vk counterpart
        self.device
            .vk_device
            .queue_submit(self.device.vk_queue, submits, fence)
    }

    #[inline(always)]
    pub unsafe fn create_fence(&self, fence_create_info: &vk::FenceCreateInfo) -> vk::Fence {
        self.device
            .vk_device
            .create_fence(fence_create_info, None)
            .expect("Failed to create Vulkan fence")
    }

    #[inline(always)]
    pub unsafe fn destroy_fence(&self, fence: vk::Fence) {
        self.device.vk_device.destroy_fence(fence, None);
    }

    #[inline(always)]
    pub unsafe fn wait_for_fences(
        &self,
        fences: &[vk::Fence],
        wait_all: bool,
        timeout: u64,
    ) -> Result<(), vk::Result> {
        self.device
            .vk_device
            .wait_for_fences(fences, wait_all, timeout)
    }

    #[inline(always)]
    pub unsafe fn reset_fences(&self, fences: &[vk::Fence]) -> Result<(), vk::Result> {
        self.device.vk_device.reset_fences(fences)
    }

    #[inline(always)]
    pub unsafe fn create_descriptor_set_layout(
        &self,
        descriptor_info: &vk::DescriptorSetLayoutCreateInfo,
    ) -> Result<vk::DescriptorSetLayout, vk::Result> {
        self.device
            .vk_device
            .create_descriptor_set_layout(descriptor_info, None)
    }

    #[inline(always)]
    pub unsafe fn destroy_descriptor_set_layout(
        &self,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) {
        self.device
            .vk_device
            .destroy_descriptor_set_layout(descriptor_set_layout, None)
    }

    #[inline(always)]
    pub unsafe fn allocate_descriptor_sets(
        &self,
        allocate_info: &vk::DescriptorSetAllocateInfo,
    ) -> Result<Vec<vk::DescriptorSet>, vk::Result> {
        self.device
            .vk_device
            .allocate_descriptor_sets(allocate_info)
    }

    #[inline(always)]
    pub unsafe fn update_descriptor_sets(
        &self,
        descriptor_writes: &[vk::WriteDescriptorSet],
        descriptor_copies: &[vk::CopyDescriptorSet],
    ) {
        self.device
            .vk_device
            .update_descriptor_sets(descriptor_writes, descriptor_copies)
    }

    #[inline(always)]
    pub unsafe fn create_descriptor_pool(
        &self,
        descriptor_pool_info: &vk::DescriptorPoolCreateInfo,
    ) -> Result<DescriptorPool, vk::Result> {
        self.device
            .vk_device
            .create_descriptor_pool(descriptor_pool_info, None)
    }

    #[inline(always)]
    pub unsafe fn destroy_descriptor_pool(&self, descriptor_pool: vk::DescriptorPool) {
        self.device
            .vk_device
            .destroy_descriptor_pool(descriptor_pool, None)
    }

    #[inline(always)]
    pub unsafe fn create_pipeline_layout(
        &self,
        layout_create_info: &vk::PipelineLayoutCreateInfo,
    ) -> Result<vk::PipelineLayout, vk::Result> {
        self.device
            .vk_device
            .create_pipeline_layout(layout_create_info, None)
    }

    #[inline(always)]
    pub unsafe fn create_graphics_pipelines(
        &self,
        pipeline_cache: vk::PipelineCache,
        graphics_pipeline_create_info: &[vk::GraphicsPipelineCreateInfo],
    ) -> Result<Vec<vk::Pipeline>, (Vec<vk::Pipeline>, vk::Result)> {
        self.device.vk_device.create_graphics_pipelines(
            pipeline_cache,
            graphics_pipeline_create_info,
            None,
        )
    }

    #[inline(always)]
    pub unsafe fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        self.device.vk_device.destroy_pipeline(pipeline, None);
    }

    #[inline(always)]
    pub unsafe fn destroy_pipeline_layout(&self, pipeline_layout: vk::PipelineLayout) {
        self.device
            .vk_device
            .destroy_pipeline_layout(pipeline_layout, None);
    }

    #[inline(always)]
    pub unsafe fn create_shader_module(
        &self,
        shader_module_create_info: &vk::ShaderModuleCreateInfo,
    ) -> Result<vk::ShaderModule, vk::Result> {
        self.device
            .vk_device
            .create_shader_module(shader_module_create_info, None)
    }

    #[inline(always)]
    pub unsafe fn destroy_shader_module(&self, shader_module: vk::ShaderModule) {
        self.device
            .vk_device
            .destroy_shader_module(shader_module, None)
    }

    // Vulkan device commands

    pub unsafe fn cmd_begin_render_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        render_pass_begin_info: &vk::RenderPassBeginInfo,
        subpass_contents: vk::SubpassContents,
    ) {
        self.device.vk_device.cmd_begin_render_pass(
            command_buffer,
            render_pass_begin_info,
            subpass_contents,
        );
    }

    pub unsafe fn cmd_end_render_pass(&self, command_buffer: vk::CommandBuffer) {
        self.device.vk_device.cmd_end_render_pass(command_buffer);
    }

    pub unsafe fn cmd_set_viewport(
        &self,
        command_buffer: vk::CommandBuffer,
        first_viewport: u32,
        viewports: &[vk::Viewport],
    ) {
        self.device
            .vk_device
            .cmd_set_viewport(command_buffer, first_viewport, viewports);
    }

    pub unsafe fn cmd_set_scissor(
        &self,
        command_buffer: vk::CommandBuffer,
        first_scissor: u32,
        scissors: &[vk::Rect2D],
    ) {
        self.device
            .vk_device
            .cmd_set_scissor(command_buffer, first_scissor, scissors);
    }

    pub unsafe fn cmd_bind_descriptor_sets(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        layout: vk::PipelineLayout,
        first_set: u32,
        descriptor_sets: &[vk::DescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        self.device.vk_device.cmd_bind_descriptor_sets(
            command_buffer,
            pipeline_bind_point,
            layout,
            first_set,
            descriptor_sets,
            dynamic_offsets,
        )
    }

    pub unsafe fn cmd_bind_pipeline(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        pipeline: vk::Pipeline,
    ) {
        self.device
            .vk_device
            .cmd_bind_pipeline(command_buffer, pipeline_bind_point, pipeline);
    }

    pub unsafe fn cmd_bind_vertex_buffers(
        &self,
        command_buffer: vk::CommandBuffer,
        first_binding: u32,
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        self.device.vk_device.cmd_bind_vertex_buffers(
            command_buffer,
            first_binding,
            buffers,
            offsets,
        )
    }

    pub unsafe fn cmd_bind_index_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        self.device
            .vk_device
            .cmd_bind_index_buffer(command_buffer, buffer, offset, index_type)
    }

    pub unsafe fn cmd_draw_indirect(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        draw_count: u32,
        stride: u32,
    ) {
        self.device
            .vk_device
            .cmd_draw_indirect(command_buffer, buffer, offset, draw_count, stride);
    }

    pub unsafe fn cmd_draw_indexed_indirect(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        draw_count: u32,
        stride: u32,
    ) {
        self.device.vk_device.cmd_draw_indexed_indirect(
            command_buffer,
            buffer,
            offset,
            draw_count,
            stride,
        );
    }

    pub unsafe fn cmd_pipeline_barrier(
        &self,
        command_buffer: vk::CommandBuffer,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        self.device.vk_device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            dependency_flags,
            memory_barriers,
            buffer_memory_barriers,
            image_memory_barriers,
        )
    }

    // Utils
    pub fn find_memory_type_index(
        &self,
        memory_requirements: &vk::MemoryRequirements,
        flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        let memory_type_count = self.device.memory_properties.memory_type_count;
        self.device.memory_properties.memory_types[..memory_type_count as _]
            .iter()
            .enumerate()
            .find(|(index, memory_type)| {
                (1 << index) & memory_requirements.memory_type_bits != 0
                    && memory_type.property_flags & flags == flags
            })
            .map(|(index, _memory_type)| index as _)
    }

    pub fn queue_family_index(&self) -> u32 {
        self.device.queue_family_index
    }
}

impl<'a> From<&'a Gpu> for &'a Device {
    fn from(gpu: &Gpu) -> &Device {
        &gpu.device
    }
}

pub struct CommandBufferIter {
    inner: std::vec::IntoIter<vk::CommandBuffer>,
}

impl Iterator for CommandBufferIter {
    type Item = vk::CommandBuffer;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl From<CommandBufferIter> for vk::CommandBuffer {
    fn from(mut iter: CommandBufferIter) -> Self {
        iter.next()
            .expect("Command buffer was not allocated: 1 of 1")
    }
}

impl From<CommandBufferIter> for (vk::CommandBuffer, vk::CommandBuffer) {
    fn from(mut iter: CommandBufferIter) -> Self {
        (
            iter.next()
                .expect("Command buffer was not allocated: 2 of 2"),
            iter.next()
                .expect("Command buffer was not allocated: 1 of 2"),
        )
    }
}

impl From<CommandBufferIter> for (vk::CommandBuffer, vk::CommandBuffer, vk::CommandBuffer) {
    fn from(mut iter: CommandBufferIter) -> Self {
        (
            iter.next()
                .expect("Command buffer was not allocated: 3 of 3"),
            iter.next()
                .expect("Command buffer was not allocated: 2 of 3"),
            iter.next()
                .expect("Command buffer was not allocated: 1 of 3"),
        )
    }
}

/// Display abstraction layer
pub struct Display {
    gpu: Gpu,
    swapchain: Arc<Swapchain>,
    surface: Surface,
    window: window::Instance,
    present_complete_semaphore: vk::Semaphore,
    render_complete_semaphore: vk::Semaphore,
    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,
}

impl Drop for Display {
    fn drop(&mut self) {
        log::debug!("Display::drop");
        unsafe {
            self.gpu.device_wait_idle().unwrap();

            Self::destroy_depth_image(
                &self.gpu,
                self.depth_image,
                self.depth_image_view,
                self.depth_image_memory,
            );
            self.gpu.destroy_semaphore(self.present_complete_semaphore);
            // NOTE: render_complete_semaphore is not owned

            Swapchain::destroy(&self.swapchain, &self.gpu);

            self.surface
                .loader
                .destroy_surface(self.surface.vk_surface, None);
        }
    }
}

impl Display {
    pub fn new(desc: DisplaySetup) -> Self {
        let vk_entry = unsafe { ash::Entry::load().expect("The environment lacks Vulkan support") };

        let vk_instance = unsafe { Self::create_instance(&desc, &vk_entry) };

        let vk_debug = if desc.debug {
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));

            let vk_debug_utils = ash::ext::debug_utils::Instance::new(&vk_entry, &vk_instance);
            let vk_debug_callback = unsafe {
                vk_debug_utils
                    .create_debug_utils_messenger(&debug_info, None)
                    .unwrap()
            };
            Some((vk_debug_utils, vk_debug_callback))
        } else {
            None
        };

        let window = desc.window_instance;

        let mut surface = unsafe { Surface::new(&window, &vk_instance, &vk_entry) };

        let (physical_device, queue_family_index) = unsafe {
            Self::select_device(&vk_instance, &surface, desc.device_type_request.as_ref())
        };

        surface.configure(physical_device);

        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let device_memory_properties =
            unsafe { vk_instance.get_physical_device_memory_properties(physical_device) };

        let extensions_names = [
            ash::khr::swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::vk::KhrPortabilitySubsetFn::NAME.as_ptr(),
        ];

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&extensions_names)
            .enabled_features(&features);

        let vk_device = unsafe {
            vk_instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Faild to create a Vulkan device")
        };
        let vk_queue = unsafe { vk_device.get_device_queue(queue_family_index, 0) };

        let gpu = Gpu {
            device: Arc::new(Device {
                vk_device,
                vk_instance,
                vk_queue,
                vk_debug,
                memory_properties: device_memory_properties,
                queue_family_index,
                _vk_entry: vk_entry,
            }),
        };

        let swapchain = unsafe { Swapchain::create(&gpu, &surface) };

        let present_complete_sempahore_create_info = vk::SemaphoreCreateInfo::default();
        let present_complete_semaphore = unsafe {
            gpu.create_semaphore(&present_complete_sempahore_create_info)
                .expect("Failed to create completion semaphore")
        };

        let (depth_image, depth_image_view, depth_image_memory) =
            unsafe { Self::create_depth_image(&gpu, surface.vk_surface_resolution) };

        Self {
            gpu,
            surface,
            window,
            swapchain: Arc::new(swapchain),
            present_complete_semaphore,
            render_complete_semaphore: vk::Semaphore::null(),
            depth_image,
            depth_image_view,
            depth_image_memory,
        }
    }

    unsafe fn create_depth_image(
        gpu: &Gpu,
        resolution: vk::Extent2D,
    ) -> (vk::Image, vk::ImageView, vk::DeviceMemory) {
        let depth_image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D16_UNORM)
            .extent(resolution.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let depth_image = unsafe { gpu.create_image(&depth_image_create_info).unwrap() };
        let depth_image_memory_req = unsafe { gpu.get_image_memory_requirements(depth_image) };
        let depth_image_memory_index = gpu
            .find_memory_type_index(
                &depth_image_memory_req,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .expect("Unable to find suitable memory index for depth image.");

        let depth_image_allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(depth_image_memory_req.size)
            .memory_type_index(depth_image_memory_index);

        let depth_image_memory =
            unsafe { gpu.allocate_memory(&depth_image_allocate_info).unwrap() };

        unsafe {
            gpu.bind_image_memory(depth_image, depth_image_memory, 0)
                .expect("Unable to bind depth image memory")
        };

        let depth_image_view_info = vk::ImageViewCreateInfo::default()
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .level_count(1)
                    .layer_count(1),
            )
            .image(depth_image)
            .format(depth_image_create_info.format)
            .view_type(vk::ImageViewType::TYPE_2D);

        let depth_image_view = unsafe { gpu.create_image_view(&depth_image_view_info).unwrap() };

        (depth_image, depth_image_view, depth_image_memory)
    }

    unsafe fn destroy_depth_image(
        gpu: &Gpu,
        depth_image: vk::Image,
        depth_image_view: vk::ImageView,
        depth_image_memory: vk::DeviceMemory,
    ) {
        gpu.destroy_image_view(depth_image_view);
        gpu.destroy_image(depth_image);
        gpu.free_memory(depth_image_memory);
    }

    pub unsafe fn recreate_depth_image(&mut self) {
        Self::destroy_depth_image(
            &self.gpu,
            self.depth_image,
            self.depth_image_view,
            self.depth_image_memory,
        );
        let (depth_image, depth_image_view, depth_image_memory) =
            Self::create_depth_image(&self.gpu, self.surface.vk_surface_resolution);
        self.depth_image = depth_image;
        self.depth_image_view = depth_image_view;
        self.depth_image_memory = depth_image_memory;
    }

    pub unsafe fn swapchain_image_views(&self) -> std::slice::Iter<vk::ImageView> {
        self.swapchain.vk_present_image_views.iter()
    }

    pub unsafe fn depth_image_view(&self) -> vk::ImageView {
        self.depth_image_view
    }

    pub unsafe fn depth_image(&self) -> vk::Image {
        self.depth_image
    }

    pub fn gpu(&self) -> Gpu {
        self.gpu.clone()
    }

    pub fn next_frame(&self) -> u32 {
        let (present_index, is_suboptimal) = unsafe {
            log::debug!("Begin acquire image");
            self.swapchain
                .loader
                .acquire_next_image(
                    self.swapchain.vk_swapchain,
                    std::u64::MAX,
                    self.present_complete_semaphore,
                    vk::Fence::null(),
                )
                .unwrap()
        };
        log::debug!(
            "End acquire image: {}, is_suboptimal: {}",
            present_index,
            is_suboptimal
        );
        present_index
    }

    // pub fn surface(&self) -> &Surface {
    //    &self.surface
    // }

    pub fn surface_format(&self) -> vk::Format {
        self.surface.vk_surface_format.format
    }

    pub fn surface_resolution(&self) -> Extent2D {
        self.surface.resolution()
    }

    pub fn surface_resize_request(&self) -> bool {
        let surface_resolution = self.surface_resolution();
        let window_resolution = self.window.resolution();
        surface_resolution != window_resolution
    }

    pub fn surface_changed(&self, surface_version: u64) -> Option<u64> {
        let current_surface_version = self.surface.version;
        if current_surface_version != surface_version {
            Some(current_surface_version)
        } else {
            None
        }
    }

    pub fn surface_version(&self) -> u64 {
        self.surface.version
    }

    pub fn resize_surface(&mut self) {
        log::debug!("Resize surface");
        let window_resolution = self.window.resolution();
        let surface_resolution = match self.surface.vk_surface_capabilities.current_extent.width {
            u32::MAX => vk::Extent2D {
                width: window_resolution.width,
                height: window_resolution.height,
            },
            _ => self.surface.vk_surface_capabilities.current_extent,
        };

        log::debug!(
            "swapchain_size: {:?}, window size: {:?}",
            surface_resolution,
            window_resolution
        );

        self.surface.vk_surface_resolution = surface_resolution;
        unsafe {
            // self.destroy_framebuffers();
            self.swapchain.destroy(&self.gpu);
            self.swapchain = Arc::new(Swapchain::create(&self.gpu, &self.surface));
            // self.framebuffers = Vulkan::create_framebuffers(
            //    &self.vk_device,
            //    &self.surface,
            //    &self.swapchain,
            //    &self.renderpass,
            //);
        }
        self.surface.version += 1;
    }

    pub fn presenter(&self, swapchain_index: u32) -> FramePresenter {
        FramePresenter {
            swapchain: Arc::clone(&self.swapchain),
            device: Arc::clone(&self.gpu.device),
            render_complete_semaphore: self.render_complete_semaphore,
            swapchain_index,
        }
    }

    pub fn set_render_complete_semaphore(&mut self, semaphore: vk::Semaphore) {
        self.render_complete_semaphore = semaphore;
    }

    pub unsafe fn present_complete_semaphore(&self) -> vk::Semaphore {
        self.present_complete_semaphore
    }

    unsafe fn create_instance(desc: &DisplaySetup, vk_entry: &ash::Entry) -> ash::Instance {
        let window = &desc.window_instance;

        //let (surface_width, surface_height) = desc.window.size();
        let raw_display_handle = window
            .display_handle()
            .expect("Could not get display handle")
            .as_raw();
        //let raw_window_handle = desc.window.handle().raw_window_handle();
        let mut extensions = ash_window::enumerate_required_extensions(raw_display_handle)
            .expect("Failed to obtain extensions requirements")
            .to_vec();

        if desc.debug {
            extensions.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extensions.push(vk::KhrPortabilityEnumerationFn::NAME.as_ptr());
            // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
            extensions.push(vk::KhrGetPhysicalDeviceProperties2Fn::NAME.as_ptr());
        }

        let app_name = CString::new(desc.app_name).ok().unwrap();
        let engine_name = CString::new(env!("CARGO_PKG_NAME")).ok().unwrap();
        let engine_version = env!("CARGO_PKG_VERSION_MAJOR")
            .parse::<u32>()
            .ok()
            .unwrap_or(0);

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(desc.app_version)
            .engine_name(&engine_name)
            .engine_version(engine_version)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        let layers_names_raw: Vec<*const c_char> = [b"VK_LAYER_KHRONOS_validation\0"]
            .iter()
            .map(|raw_name| unsafe { CStr::from_bytes_with_nul_unchecked(*raw_name).as_ptr() })
            .collect();

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extensions)
            .flags(create_flags);

        vk_entry
            .create_instance(&create_info, None)
            .expect("Could not create a Vulkan instanceon error")
    }

    /// Returns device instance that sattisfies requirements
    unsafe fn select_device(
        vk_instance: &ash::Instance,
        surface: &Surface,
        device_type_request: Option<&DeviceType>,
    ) -> (vk::PhysicalDevice, u32) {
        let vk_devices = vk_instance
            .enumerate_physical_devices()
            .expect("Failed to detect GPUs");

        vk_devices
            .iter()
            .find_map(|p_device| {
                unsafe {
                    let device_properties = vk_instance.get_physical_device_properties(*p_device);

                    let device_type = device_properties.device_type;

                    if let Some(device_type_request) = device_type_request.as_ref() {
                        match *device_type_request {
                            DeviceType::Integrated => {
                                if device_type != vk::PhysicalDeviceType::INTEGRATED_GPU {
                                    return None;
                                }
                            }
                            DeviceType::Discrete => {
                                if device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
                                    return None;
                                }
                            }
                        }
                    }

                    vk_instance
                        .get_physical_device_queue_family_properties(*p_device)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            // if !info.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                            //    return None;
                            // }
                            if !info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                                return None;
                            }
                            if !surface.get_physical_device_support(*p_device, index as u32) {
                                return None;
                            }

                            let device_name =
                                CStr::from_ptr(device_properties.device_name.as_ptr());

                            log::info!("Select GPU: {:?} ({:?})", device_name, device_type);
                            Some((*p_device, index as u32))
                        })
                }
            })
            .expect("Could not find a device that fulfill requirements")
    }

    /*
    pub fn present(&self) {
        let present_index = self.next_frame();
        log::info!("present_index {}", present_index);
        let clear_values = vec![
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 1.0, 1.0, 0.0],
                },
            },
            //vk::ClearValue {
            //    depth_stencil: vk::ClearDepthStencilValue {
            //        depth: 1.0,
            //       stencil: 0,
            //    },
            //},
        ];

        // these array must live long enough or vulkan will sigsegv
        let wait_semaphores = [self.vk_present_complete_semaphore];
        let signal_semaphores = [self.vk_render_complete_semaphore];
        let command_buffers = [self.vk_draw_command_buffer];

        log::info!("render_pass_begin_info");
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass)
            .framebuffer(self.framebuffers[present_index as usize])
            .render_area(self.surface.vk_resolution.into())
            .clear_values(&clear_values)
            .build();

        log::info!("begin submit");
        // begin submit
        unsafe {
            self.vk_device
                .wait_for_fences(&[self.vk_draw_commands_reuse_fence], true, u64::MAX)
                .expect("Wait for fence failed.");

            log::info!("wait_for_fences");
            self.vk_device
                .reset_fences(&[self.vk_draw_commands_reuse_fence])
                .expect("Reset fences failed.");

            log::info!("reset_fences");
            self.vk_device
                .reset_command_buffer(
                    self.vk_draw_command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Reset command buffer failed.");

            log::info!("reset_command_buffer");
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();

            log::info!("command_buffer_begin_info");
            self.vk_device
                .begin_command_buffer(self.vk_draw_command_buffer, &command_buffer_begin_info)
                .expect("Begin commandbuffer");
            log::info!("begin_command_buffer");
        }
        // ------------
        unsafe {
            self.vk_device.cmd_begin_render_pass(
                self.vk_draw_command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            log::info!("cmd_begin_render_pass");
            self.vk_device
                .cmd_end_render_pass(self.vk_draw_command_buffer);
        }
        log::info!("cmd_end_render_pass");
        // ------------
        unsafe {
            self.vk_device
                .end_command_buffer(self.vk_draw_command_buffer)
                .expect("End commandbuffer");
            log::info!("end_command_buffer");

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores)
                .build();
            log::info!("submit_info {:?}", self.vk_queue);

            self.vk_device
                .queue_submit(
                    self.vk_queue,
                    &[submit_info],
                    self.vk_draw_commands_reuse_fence,
                )
                .expect("queue submit failed.");
            log::info!("queue_submit");
        }
        // end submit

        let wait_semaphores = [self.vk_render_complete_semaphore];
        let swapchains = [self.swapchain.vk_swapchain];
        let image_indices = [present_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[self.vk_render_complete_semaphore])
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .build();
        log::info!("present_info");

        unsafe {
            self.swapchain
                .loader
                .queue_present(self.vk_queue, &present_info)
                .unwrap();
        }
        log::info!("queue_present");
    }
    */

    /*
    unsafe fn create_framebuffers(
        device: &ash::Device,
        surface: &Surface,
        swapchain: &Swapchain,
        renderpass: &vk::RenderPass,
    ) -> Vec<vk::Framebuffer> {
        swapchain
            .vk_present_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view /*, base.depth_image_view*/];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(*renderpass)
                    .attachments(&framebuffer_attachments)
                    .width(surface.vk_resolution.width)
                    .height(surface.vk_resolution.height)
                    .layers(1)
                    .build();

                device
                    .create_framebuffer(&frame_buffer_create_info, None)
                    .expect("Could not create a framebuffer")
            })
            .collect::<Vec<_>>()
    }

    unsafe fn destroy_framebuffers(&mut self) {
        for framebuffer in self.framebuffers.iter() {
            self.vk_device.destroy_framebuffer(*framebuffer, None);
        }
        self.framebuffers.clear();
    }
    */
}

impl<'a> From<&'a Display> for &'a Device {
    fn from(display: &Display) -> &Device {
        &display.gpu.device
    }
}

pub struct FramePresenter {
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
    render_complete_semaphore: vk::Semaphore,
    swapchain_index: u32,
}

impl FramePresenter {
    pub fn present(self) {
        let wait_semaphores = [self.render_complete_semaphore];
        let swapchains = [self.swapchain.as_ref().vk_swapchain];
        let image_indices = [self.swapchain_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        log::debug!("Begin present: {}", self.swapchain_index);

        unsafe {
            let r = self
                .swapchain
                .loader
                .queue_present(self.device.vk_queue, &present_info);
            log::debug!("end present: {:?}", r);
            r.unwrap();
        }
    }
}

/*
pub struct CommandRecorder<'a> {
    gpu: &'a Gpu,
    command_buffer: vk::CommandBuffer,
    one_time_submit: bool,
}

impl<'a> Drop for CommandRecorder<'a> {
    fn drop(&mut self) {
        if self.one_time_submit {
            unsafe {
                self.gpu.end_command_buffer(self.command_buffer);
            }
        }
    }
}

impl<'a> CommandRecorder<'a> {
    pub fn setup() -> CommandRecorderSetup {
        CommandRecorderSetup {
            one_time_submit: true,
            ..Default::default()
        }
    }


}

#[derive(Default)]
pub struct CommandRecorderSetup {
    pub command_buffer: vk::CommandBuffer,
    pub reuse_fence: Option<vk::Fence>,
    pub one_time_submit: bool,
}

impl CommandRecorderSetup {
    #[inline(always)]
    pub fn command_buffer(mut self, command_buffer: vk::CommandBuffer) -> Self {
        self.command_buffer = command_buffer;
        self
    }
    #[inline(always)]
    pub fn reuse_fence(mut self, reuse_fence: Option<vk::Fence>) -> Self {
        self.reuse_fence = reuse_fence;
        self
    }
    #[inline(always)]
    pub fn one_time_submit(mut self, one_time_submit: bool) -> Self {
        self.one_time_submit = one_time_submit;
        self
    }

    pub fn create<'a>(self, gpu: &'a Gpu) -> CommandRecorder<'a> {
        unsafe {
            if let Some(reuse_fence) = self.reuse_fence {
                gpu.wait_for_fences(&[reuse_fence], true, u64::MAX)
                    .expect("Failed to wait for Vulkan fences");
                gpu.reset_fences(&[reuse_fence])
                    .expect("Failed to reset Vulkan fences");
            }

            if self.one_time_submit {
                let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                gpu.reset_command_buffer(
                    self.command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Failed to reset Vulkan command buffer");

                gpu.begin_command_buffer(self.command_buffer, &command_buffer_begin_info);
            }
        }

        CommandRecorder {
            gpu,
            command_buffer: self.command_buffer,
            one_time_submit: self.one_time_submit,
        }
    }
}
*/

pub struct Buffer {
    pub handle: vk::Buffer,
    pub device_memory: vk::DeviceMemory,
    pub size: u64,
}

impl Buffer {
    pub unsafe fn create_and_allocate(
        gpu: &Gpu,
        buffer_create_info: &vk::BufferCreateInfo,
    ) -> Result<Buffer, vk::Result> {
        let buffer = gpu.create_buffer(&buffer_create_info)?;

        let buffer_memory_req = gpu.get_buffer_memory_requirements(buffer);

        let buffer_memory_index = gpu
            .find_memory_type_index(
                &buffer_memory_req,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("Unable to find suitable memorytype for the buffer.");

        let buffer_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: buffer_memory_req.size,
            memory_type_index: buffer_memory_index,
            ..Default::default()
        };

        let buffer_memory = gpu.allocate_memory(&buffer_allocate_info)?;

        gpu.bind_buffer_memory(buffer, buffer_memory, 0)?;

        Ok(Buffer {
            handle: buffer,
            device_memory: buffer_memory,
            size: buffer_memory_req.size,
        })
    }

    pub unsafe fn map_and_write_to_device_memory<T: Copy>(
        &self,
        gpu: &Gpu,
        offset: u64,
        data: &[T],
    ) {
        let align = std::mem::align_of::<T>() as u64;
        let size = (data.len() * std::mem::size_of::<T>()) as u64;

        let memory_ptr = gpu
            .map_memory(
                self.device_memory,
                offset,
                size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Could not map buffer memory");

        let mut index_slice = ash::util::Align::new(memory_ptr, align, size);
        index_slice.copy_from_slice(data);
        gpu.unmap_memory(self.device_memory);
    }

    pub unsafe fn free_memory_and_destroy(&self, gpu: &Gpu) {
        gpu.device_wait_idle().expect("Device is not idle");
        gpu.free_memory(self.device_memory);
        gpu.destroy_buffer(self.handle);
    }
}

/*

pub struct PipelineLayout {
    vk_pipeline_layout: vk::PipelineLayout,
    device: Arc<Device>,
}

#[derive(Default)]
pub struct PipelineLayoutSetup<'a> {
    vk_layout_create_info: vk::PipelineLayoutCreateInfo<'a>,
}

impl PipelineLayout {
    pub fn setup<'a>() -> PipelineLayoutSetup<'a> {
        PipelineLayoutSetup::default()
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_device
                .destroy_pipeline_layout(self.vk_pipeline_layout, None);
        };
    }
}

pub struct GraphicsPipeline {
    vk_pipeline: vk::Pipeline,
    device: Arc<Device>,
}

impl GraphicsPipeline {
    pub fn setup<'a>() -> GraphicsPipelineSetup<'a> {
        GraphicsPipelineSetup::default()
    }
}

#[derive(Default)]
pub struct GraphicsPipelineSetup<'a> {
    vk_graphics_pipline_create_info: vk::GraphicsPipelineCreateInfo<'a>,
}

impl<'a> GraphicsPipelineSetup<'a> {
    pub fn create(self, gpu: &Gpu) -> GraphicsPipeline {
        gpu.create_graphics_pipeline(self.vk_graphics_pipline_create_info)
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_device
                .destroy_pipeline(self.vk_pipeline, None);
        };
    }
}

pub struct ShaderModule {
    device: Arc<Device>,
    vk_shader_module: vk::ShaderModule,
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_device
                .destroy_shader_module(self.vk_shader_module, None);
        };
    }
}

 */

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::debug!(
            "{message_type:?}|{message_id_name}|{message_id_number}: {message}\n",
        ),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::info!(
            "{message_type:?}|{message_id_name}|{message_id_number}: {message}\n",
        ),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!(
            "{message_type:?}|{message_id_name}|{message_id_number}: {message}\n",
        ),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::error!(
            "{message_type:?}|{message_id_name}|{message_id_number}: {message}\n",
        ),
        _ => log::debug!(
            "{message_severity:?}|{message_type:?}|{message_id_name}|{message_id_number}: {message}\n",
        ),
    }

    vk::FALSE
}
