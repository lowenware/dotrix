use std::time::{Duration, Instant};
use winit::window::Window;
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    surface: wgpu::Surface,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropiate adapter");

        // Create the logical device and command queue
        // TODO: research available features
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            // TODO: Allow srgb unconditionally
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        Self {
            device,
            queue,
            sc_desc,
            surface,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn sc_desc(&self) -> &wgpu::SwapChainDescriptor {
        &self.sc_desc
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.sc_desc.width = width;
        self.sc_desc.height = height;
    }

    pub fn swap_chain(&self) -> wgpu::SwapChain {
        self.device.create_swap_chain(&self.surface, &self.sc_desc)
    }
}
