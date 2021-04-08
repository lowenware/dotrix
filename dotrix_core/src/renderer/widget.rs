use bytemuck::{ Pod, Zeroable };

use crate::{
    assets::{
        Id,
        Texture,
        VertexAttributes,
    },
    renderer::pipeline::Pipeline,
    services::{
        Assets,
        Renderer,
        Window,
    },
};

/// Overlay widget component 
pub struct Widget {
    /// list of vertices
    pub vertices: Vec<WidgetVertex>,
    /// list of indices
    pub indices: Option<Vec<u32>>,
    /// custom [`Id`] of an overlay [`Texture`]
    pub texture: Id<Texture>,
    /// custom [`Id`] of a rendering [`Pipeline`]
    pub pipeline: Id<Pipeline>,
    /// Minimal clip size by X axis
    pub clip_min_x: u32,
    /// Minimal clip size by Y axis
    pub clip_min_y: u32,
    /// widget width
    pub width: u32,
    /// widget height
    pub height: u32,
    /// Pipeline buffers
    pub buffers: Option<Buffers>,
}

/// Pipeline buffers
pub struct Buffers {
    bind_group: wgpu::BindGroup,
    vertices_buffer: wgpu::Buffer,
    indices_buffer: Option<wgpu::Buffer>,
    screen_size: wgpu::Buffer,
}

impl Widget {
    /// Returns loaded assets if they are all ready
    fn get_texture<'a>(
        &self,
        assets: &'a mut Assets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<&'a Texture, ()> {

        if let Some(texture) = assets.get_mut(self.texture) {
            texture.load(device, queue);
        }

        if let Some(texture) = assets.get(self.texture) {
            return Ok(texture);
        }

        Err(())
    }

    /// Loads [`Widget`] data to buffers
    pub(crate) fn load(
        &mut self,
        renderer: &Renderer,
        assets: &mut Assets,
        pipeline: &Pipeline,
        sampler: &wgpu::Sampler,
        window: &Window,
    ) {
        use wgpu::util::DeviceExt;

        let device = &renderer.device;
        let queue = &renderer.queue;

        let screen_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let screen_size = [
            screen_size.x as f32 / scale_factor,
            screen_size.y as f32 / scale_factor
        ];

        if let Some(buffers) = self.buffers.as_ref() {
            queue.write_buffer(&buffers.screen_size, 0, bytemuck::cast_slice(&screen_size));
        } else {
            self.buffers = if let Ok(texture) = self.get_texture(assets, device, queue) {

                let vertices_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Overlay Vertex Buffer"),
                        contents: bytemuck::cast_slice(&self.vertices),
                        usage: wgpu::BufferUsage::VERTEX,
                    }
                );

                let indices_buffer = self.indices.as_ref().map(|indices| {
                    device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Overlay Indices Buffer"),
                            contents: bytemuck::cast_slice(indices),
                            usage: wgpu::BufferUsage::INDEX,
                        }
                    )
                });

                let screen_size = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Overlay screen_size"),
                    contents: bytemuck::cast_slice(&screen_size),
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &pipeline.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: screen_size.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(texture.view()),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                    label: None,
                });

                Some(
                    Buffers {
                        bind_group,
                        vertices_buffer,
                        indices_buffer,
                        screen_size,
                    }
                )
            } else {
                None
            };
        }
    }

    /// Renders widget
    pub(crate) fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &Pipeline,
        frame: &wgpu::SwapChainTexture,
    ) {
        if let Some(buffers) = self.buffers.as_ref() {

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.push_debug_group("Prepare to draw an Overlay");
            rpass.set_pipeline(&pipeline.wgpu_pipeline);
            rpass.set_scissor_rect(self.clip_min_x, self.clip_min_y, self.width, self.height);
            rpass.set_bind_group(0, &buffers.bind_group, &[]);
            rpass.set_vertex_buffer(0, buffers.vertices_buffer.slice(..));
            rpass.pop_debug_group();

            if let Some(indices_buffer) = buffers.indices_buffer.as_ref() {
                let indices_count = self.indices.as_ref().unwrap().len() as u32;
                rpass.insert_debug_marker("Draw indexed model");
                rpass.set_index_buffer(indices_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..indices_count, 0, 0..1);
            } else {
                rpass.insert_debug_marker("Draw an overlay");
                rpass.draw(0..self.vertices.len() as u32, 0..1);
            }
        }
    }
}

impl Default for Widget {
    fn default() -> Self {
        Widget {
            vertices: Vec::new(),
            indices: None,
            texture: Id::default(),
            pipeline: Id::default(),
            buffers: None,
            clip_min_x: 0,
            clip_min_y: 0,
            width: 640,
            height: 480,
        }
    }
}

/// Vertex buffer element
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WidgetVertex {
    /// Vertex position
    pub position: [f32; 2],
    /// Vertex texture coordinate
    pub uv: [f32; 2],
    /// Vertex color
    pub color: [f32; 4],
}

unsafe impl Pod for WidgetVertex {}
unsafe impl Zeroable for WidgetVertex {}
impl VertexAttributes for WidgetVertex {}
