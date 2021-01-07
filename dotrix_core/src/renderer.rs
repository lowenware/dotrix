pub mod bind_group_layout;
mod light;
pub mod pipeline;
pub mod skybox;
mod model;
mod overlay;
mod widget;

pub mod transform;
pub use transform::*;
pub use model::*;
pub use skybox::*;
pub use light::{ Light, LightUniform };
pub use overlay::{ Overlay, overlay_update, Provider as OverlayProvider };
pub use widget::{ Widget, WidgetVertex };

use pipeline::Pipeline;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use dotrix_math::{Mat4, Deg, perspective};

use crate::{
    assets::Id,
    ecs::{ Const, Mut, Context },
    services::{ Assets, Camera, World },
};

pub struct Renderer {
    pub window: winit::window::Window,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub depth_buffer: wgpu::TextureView,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub clear_color: wgpu::Color,
    pub frame: Option<wgpu::SwapChainFrame>,
    pub projection: Mat4,
    pub pipelines: HashMap<Id<Pipeline>, Pipeline>,
    pub overlay: Vec<Overlay>,
}

pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Renderer {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface,
        window: winit::window::Window,
        clear_color: [f64; 4],
    ) -> Self {
        let size = window.inner_size();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let depth_buffer = Self::create_depth_buffer(&device, size.width, size.height);
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let clear_color = wgpu::Color { 
            r: clear_color[0],
            g: clear_color[1],
            b: clear_color[2],
            a: clear_color[3],
        };

        Self {
            window,
            device,
            queue,
            surface,
            depth_buffer,
            sc_desc,
            swap_chain,
            clear_color,
            frame: None,
            projection: Self::frustum(size.width as f32 / size.height as f32),
            pipelines: HashMap::new(),
            overlay: Vec::new(),
        }
    }

    pub fn add_overlay(&mut self, overlay_provider: Box<dyn OverlayProvider>) {
        self.overlay.push(Overlay::new(overlay_provider));
    }

    pub fn overlay_provider<T: 'static + Send + Sync>(&self) -> Option<&T> {
        for overlay in &self.overlay {
            let provider = overlay.provider::<T>();
            if provider.is_some() {
                return provider;
            }
        }
        None
    }

    pub fn add_pipeline(&mut self, pipeline: Pipeline) -> Id<Pipeline> {
        let id = Id::new(self.pipelines.len() as u64 + 1);
        self.pipelines.insert(id, pipeline);
        id
    }

    pub fn pipeline(&self, id: Id<Pipeline>) -> &Pipeline {
        self.pipelines.get(&id).expect("Pipeline has to be registered with `add_pipeline` method")
    }

    pub fn add_skybox_pipeline(&mut self) -> Id<Pipeline> {
        let pipeline = Pipeline::default_for_skybox(&self.device, &self.sc_desc);
        self.add_pipeline(pipeline)
    }

    pub fn add_static_model_pipeline(&mut self) -> Id<Pipeline> {
        let pipeline = Pipeline::default_for_static_model(&self.device, &self.sc_desc);
        self.add_pipeline(pipeline)
    }

    pub fn add_skinned_model_pipeline(&mut self) -> Id<Pipeline> {
        let pipeline = Pipeline::default_for_skinned_model(&self.device, &self.sc_desc);
        self.add_pipeline(pipeline)
    }

    pub fn add_overlay_pipeline(&mut self) -> Id<Pipeline> {
        let pipeline = Pipeline::default_for_overlay(&self.device, &self.sc_desc);
        self.add_pipeline(pipeline)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.sc_desc.width = width;
            self.sc_desc.height = height;

            self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
            self.depth_buffer = Self::create_depth_buffer(&self.device, width, height);

            let aspect_ratio = width as f32 / height as f32;
            let projection = Self::frustum(aspect_ratio);
            self.projection = projection;
        }
    }

    pub fn frame(&self) -> Option<&wgpu::SwapChainFrame> {
        self.frame.as_ref()
    }

    pub fn next_frame(&mut self) {
        let frame = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
                self.swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture!")
            }
        };
        self.frame = Some(frame);
    }

    pub fn finalize(&mut self) {
        self.frame.take();
    }

    pub fn display_size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        ( size.width, size.height )
    }

    pub fn display_virtual_size(&self) -> (f32, f32) {
        let size = self.window.inner_size();
        let scale_factor = self.window.scale_factor() as f32;
        ( size.width as f32 / scale_factor, size.height as f32 / scale_factor )
    }

    pub fn scale_factor(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    fn create_depth_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        let buffer_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = wgpu::TextureDescriptor {
            label: Some("Depth Buffer"),
            size: buffer_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::RENDER_ATTACHMENT,
        };

        device
            .create_texture(&texture)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn frustum(aspect_ratio: f32) -> Mat4 {
        let fov = Deg(70f32);
        let near_plane = 0.1;
        let far_plane = 2000.0;

        perspective(fov, aspect_ratio, near_plane, far_plane)
    }
}

/// Default render pipelines provided by the engine
pub struct Pipelines {
    skybox: Id<Pipeline>,
    static_model: Id<Pipeline>,
    skinned_model: Id<Pipeline>,
    overlay: Id<Pipeline>,
}

#[derive(Default)]
pub struct WorldRenderer {
    lights_buffer: Option<wgpu::Buffer>,
    proj_view_buffer: Option<wgpu::Buffer>,
    sampler: Option<wgpu::Sampler>,
    pipelines: Option<Pipelines>,
}

pub fn world_renderer(
    mut ctx: Context<WorldRenderer>,
    mut renderer: Mut<Renderer>,
    mut assets: Mut<Assets>,
    camera: Const<Camera>,
    world: Const<World>
) {
    if ctx.pipelines.is_none() {
        let skybox = renderer.add_skybox_pipeline();
        let static_model = renderer.add_static_model_pipeline();
        let skinned_model = renderer.add_skinned_model_pipeline();
        let overlay = renderer.add_overlay_pipeline();
        ctx.pipelines = Some(
            Pipelines {
                skybox,
                static_model,
                skinned_model,
                overlay,
            }
        );
    }

    let device = &renderer.device;
    let queue = &renderer.queue;
    let depth_buffer = &renderer.depth_buffer;
    let frame = &renderer.frame()
        .expect("Frame should be created before the rendering cycle")
        .output;

    // Prepare sampler
    if ctx.sampler.is_none() {
        ctx.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));
    }

    // Prepare lights
    let query = world.query::<(&mut Light,)>();
    let mut lights = LightUniform::default();
    for (light,) in query {
        lights.push(*light);
    }

    if let Some(lights_buffer) = ctx.lights_buffer.as_ref() {
        queue.write_buffer(lights_buffer, 0, bytemuck::cast_slice(&[lights]));
    } else {
        ctx.lights_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[lights]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        ));
    }

    // Prepare projection * view matrix
    let proj_view_matrix = OPENGL_TO_WGPU_MATRIX * renderer.projection * camera.view();
    let proj_view_slice = AsRef::<[f32; 16]>::as_ref(&proj_view_matrix);

    if let Some(proj_view_buffer) = ctx.proj_view_buffer.as_ref() {
        queue.write_buffer(proj_view_buffer, 0, bytemuck::cast_slice(proj_view_slice));
    } else {
        ctx.proj_view_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ProjView Buffer"),
            contents: bytemuck::cast_slice(proj_view_slice),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        }));
    }

    // prepare the command encoder and clean the surface
    let command_encoder_descriptor = wgpu::CommandEncoderDescriptor { label: None };
    let mut encoder = device.create_command_encoder(&command_encoder_descriptor);
    {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(renderer.clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: depth_buffer,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
    }

    let sampler = ctx.sampler.as_ref().unwrap();

    // render skybox
    let query = world.query::<(&mut SkyBox,)>();
    for (skybox,) in query {
        if skybox.pipeline.is_null() {
            skybox.pipeline = ctx.pipelines.as_ref().unwrap().skybox;
        }
        let pipeline = renderer.pipeline(skybox.pipeline);
        let proj_view = OPENGL_TO_WGPU_MATRIX * renderer.projection * camera.view_static();

        skybox.load(&assets, device, queue, pipeline, sampler, &proj_view);
        skybox.draw(&mut encoder, pipeline, frame);
    }

    // render static models
    let query = world.query::<(&mut Model,)>();
    for (model,) in query {
        if model.disabled {
            continue;
        }

        if model.pipeline.is_null() {
            let pipelines = ctx.pipelines.as_ref().unwrap();
            model.pipeline = if !model.skin.is_null() {
                pipelines.skinned_model
            } else {
                pipelines.static_model
            };
        }
        let pipeline = renderer.pipeline(model.pipeline);
        let proj_view_buffer = ctx.proj_view_buffer.as_ref().unwrap();
        let lights_buffer = ctx.lights_buffer.as_ref().unwrap();

        model.load(&renderer, &mut assets, pipeline, sampler, proj_view_buffer, lights_buffer);
        model.draw(&assets, &mut encoder, pipeline, frame, depth_buffer);
    }

    for overlay in &renderer.overlay {
        let scale_factor = renderer.window.scale_factor() as f32;
        let size = renderer.window.inner_size();

        for widget in &mut overlay.widgets(scale_factor, size.width as f32, size.height as f32) {
            if widget.pipeline.is_null() {
                widget.pipeline = ctx.pipelines.as_ref().unwrap().overlay;
            }
            let pipeline = renderer.pipeline(widget.pipeline);
            widget.load(&renderer, &mut assets, pipeline, sampler);
            widget.draw(&mut encoder, pipeline, frame);
        }
    }

    // submit rendering
    queue.submit(Some(encoder.finish()));
}


