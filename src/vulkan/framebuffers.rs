use super::{Device, Gpu, RenderPass};
use crate::render::Extent2D;
use ash::vk;
use std::sync::Arc;

pub struct Framebuffers {
    vk_framebuffers: Vec<vk::Framebuffer>,
    surface_version: u64,
}

impl Framebuffers {
    pub fn new() -> Self {
        Self {
            vk_framebuffers: vec![],
            surface_version: 0,
        }
    }

    pub fn set(&mut self, display: &Display, render_pass: vk::RenderPass) {
        let surface_version = display.surface_version();
        if self.surface_version != surface_version {
            let resolution = display.surface_resolution();
            self.surface_version = surface_version;
            unsafe {
                self.destroy(&display.device);
                self.create(display, render_pass);
            }
        }
    }

    unsafe fn create(&mut self, display: &Display, render_pass: vk::RenderPass) {
        let resolution = display.surface_resolution();
        self.vk_framebuffers = display
            .swapchain
            .vk_present_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view /*, base.depth_image_view*/];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass.vk_render_pass)
                    .attachments(&framebuffer_attachments)
                    .width(resolution.width)
                    .height(resolution.height)
                    .layers(1)
                    .build();

                display
                    .device
                    .vk_device
                    .create_framebuffer(&frame_buffer_create_info, None)
                    .expect("Could not create a framebuffer")
            })
            .collect::<Vec<_>>()
    }

    unsafe fn destroy(&self, device: Into<&Device>) {
        for framebuffer in self.vk_framebuffers.iter() {
            device
                .into()
                .vk_device
                .destroy_framebuffer(*framebuffer, None);
        }
    }
}
