use bytemuck::{ Pod, Zeroable };
use cgmath::{
    Vector2,
    Vector3,
};
use crate::{
    assets::{
        Id,
        Texture,
        VertexAttributes,
    },
    services::{
        Assets,
        Renderer,
    },
    math::Transform,
};
use super::pipeline::Pipeline;

#[derive(Default)]
pub struct Overlay {
    pub widgets: Vec<Widget>,
}

impl Overlay {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

pub struct Buffers {
    pub bind_group: wgpu::BindGroup,
    pub vertices_buffer: wgpu::Buffer,
    pub transform: wgpu::Buffer,
}

pub struct Widget {
    pub positions: Vec<[f32; 2]>,
    pub uvs: Vec<[f32; 2]>,
    pub texture: Id<Texture>,
    pub pipeline: Id<Pipeline>,
    pub translate: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub buffers: Option<Buffers>,
}

impl Widget {
    pub fn vertices(&self) -> Result<Vec<OverlayVertex>, ()> {
        if self.positions.len() == self.uvs.len() {
            let result = self.positions
                .iter()
                .zip(self.uvs.iter())
                .map(|(&position, &uv)| OverlayVertex { position, uv })
                .collect::<Vec<_>>();
            Ok(result)
        } else {
            Err(())
        }
    }

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

    pub(crate) fn load(
        &mut self,
        renderer: &Renderer,
        assets: &mut Assets,
        pipeline: &Pipeline,
        sampler: &wgpu::Sampler,
    ) {
        use wgpu::util::DeviceExt;

        let device = renderer.device();
        let queue = renderer.queue();

        let transform_matrix = (Transform {
            translate: Vector3::new(self.translate.x, self.translate.y, 0.0),
            scale: Vector3::new(self.scale.x, self.scale.y, 1.0),
            ..Default::default()
        }).matrix();
        let transform = AsRef::<[f32; 16]>::as_ref(&transform_matrix);

        if let Some(buffers) = self.buffers.as_ref() {
            queue.write_buffer(&buffers.transform, 0, bytemuck::cast_slice(transform));
        } else {
            self.buffers = if let Ok(texture) = self.get_texture(assets, device, queue) {

                let vertices = self.vertices().expect("Overlay needs vertices data");
                let vertices_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Overlay Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsage::VERTEX,
                    }
                );

                let transform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Overlay Transform"),
                    contents: bytemuck::cast_slice(transform),
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &pipeline.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: transform.as_entire_binding(),
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
                        transform,
                    }
                )
            } else {
                None
            };
        }
    }

    pub(crate) fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &Pipeline,
        frame: &wgpu::SwapChainTexture,
    ) {
        if let Some(buffers) = self.buffers.as_ref() {

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            rpass.set_bind_group(0, &buffers.bind_group, &[]);
            rpass.set_vertex_buffer(0, buffers.vertices_buffer.slice(..));
            rpass.pop_debug_group();

            rpass.insert_debug_marker("Draw a model");
            rpass.draw(0..self.positions.len() as u32, 0..1);
        }
    }
}

impl Default for Widget {
    fn default() -> Self {
        Widget {
            positions: vec![[-1.0, 1.0], [-1.0, -1.0], [1.0, 1.0], [1.0, -1.0]],
            uvs: vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
            texture: Id::default(),
            pipeline: Id::default(),
            translate: Vector2::new(0.0, 0.0),
            scale: Vector2::new(1.0, 1.0),
            buffers: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OverlayVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

unsafe impl Pod for OverlayVertex {}
unsafe impl Zeroable for OverlayVertex {}
impl VertexAttributes for OverlayVertex {}
