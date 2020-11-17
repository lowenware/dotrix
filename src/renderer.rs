mod light;
mod r#static;

pub use r#static::*;
pub use light::{Light, LightUniform};

use winit::window::Window;

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,
    surface: wgpu::Surface,
    frame: Option<wgpu::SwapChainFrame>,
    projection: cgmath::Matrix4<f32>,
    depth_buffer: wgpu::TextureView,
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
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
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let aspect_ratio = sc_desc.width as f32 / sc_desc.height as f32;
        let projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 400.0);
        let depth_buffer = Self::create_depth_buffer(&device, sc_desc.width, sc_desc.height);

        Self {
            device,
            queue,
            sc_desc,
            surface,
            swap_chain,
            frame: None,
            projection: OPENGL_TO_WGPU_MATRIX * projection,
            depth_buffer,
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
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        let aspect_ratio = width as f32 / height as f32;
        let projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 400.0);
        self.projection = OPENGL_TO_WGPU_MATRIX * projection;
        self.depth_buffer = Self::create_depth_buffer(&self.device, width, height);
    }

    pub fn frame(&self) -> Option<&wgpu::SwapChainFrame> {
        self.frame.as_ref()
    }

    pub fn finalize(&mut self) {
        self.frame.take();
    }

    pub fn next_frame(&mut self) {
        let frame = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
            println!("SwapChain: next frame");
                self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
                self.swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture!")
            }
        };
        self.frame = Some(frame);
    }

    pub fn projection(&self) -> &cgmath::Matrix4<f32> {
        &self.projection
    }

    pub fn depth_buffer(&self) -> &wgpu::TextureView {
        &self.depth_buffer
    }

    fn create_depth_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        let buffer_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let draw_depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Buffer"),
            size: buffer_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        draw_depth_buffer.create_view(&wgpu::TextureViewDescriptor::default())
    }
}
