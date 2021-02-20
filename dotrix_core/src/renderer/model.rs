use crate::{
    assets::{ Id, Mesh, Skin, Pose, Texture },
    services::{ Assets, Renderer },
};

use super::pipeline::Pipeline;
use super::transform::Transform;

pub struct Buffers {
    bind_group: wgpu::BindGroup,
    transform: wgpu::Buffer,
}

#[derive(Default)]
pub struct Model {
    pub mesh: Id<Mesh>,
    pub texture: Id<Texture>,
    pub transform: Transform,
    pub skin: Id<Skin>,
    pub pose: Option<Pose>,
    pub buffers: Option<Buffers>,
    pub pipeline: Id<Pipeline>,
    pub disabled: bool,
}

impl Model {

    /// Returns loaded assets if they are all ready
    fn get_assets<'a>(
        &self,
        assets: &'a mut Assets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(&'a Mesh, &'a Texture, Option<&'a Skin>), ()> {

        if !self.skin.is_null() && assets.get(self.skin).is_none() {
            return Err(());
        }

        if let Some(mesh) = assets.get_mut(self.mesh) {
            if self.skin.is_null() {
                mesh.load_as_static(device);
            } else {
                mesh.load_as_skinned(device);
            }
        }

        if let Some(texture) = assets.get_mut(self.texture) {
            texture.load(device, queue);
        }

        if let Some(mesh) = assets.get(self.mesh) {
            if let Some(texture) = assets.get(self.texture) {
                let skin = assets.get(self.skin);
                return Ok((mesh, texture, skin));
            }
        }
        Err(())
    }

    /// Initialize model specific buffers, should be called just once by renderer
    pub(crate) fn load(
        &mut self,
        renderer: &Renderer,
        assets: &mut Assets,
        pipeline: &Pipeline,
        sampler: &wgpu::Sampler,
        proj_view: &wgpu::Buffer,
        lights_buffer: &wgpu::Buffer,
    ) {
        use wgpu::util::DeviceExt;

        let device = &renderer.device;
        let queue = &renderer.queue;

        let transform_matrix = self.transform.matrix();
        let model_transform = AsRef::<[f32; 16]>::as_ref(&transform_matrix);

        if let Ok((_, texture, skin)) = self.get_assets(assets, device, queue) {
            if let Some(buffers) = self.buffers.as_ref() {
                queue.write_buffer(&buffers.transform, 0, bytemuck::cast_slice(model_transform));

                if let Some(pose) = self.pose.as_ref() {
                    if let Some(skin) = assets.get(self.skin) {
                        pose.load(&skin.index, queue);
                    }
                }
            } else {
                let transform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Model Transform"),
                    contents: bytemuck::cast_slice(model_transform),
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                });

                let mut bind_group_entries = Vec::with_capacity(6);
                let pose = skin.map(|_| Pose::new(device));
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 0,
                    resource: proj_view.as_entire_binding(),
                });
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: transform.as_entire_binding(),
                });
                if let Some(pose) = pose.as_ref() {
                    bind_group_entries.push(wgpu::BindGroupEntry {
                        binding: 2,
                        resource: pose.buffer.as_entire_binding(),
                    });
                }
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(texture.view()),
                });
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(sampler),
                });
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: 5,
                    resource: lights_buffer.as_entire_binding(),
                });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &pipeline.bind_group_layout,
                    entries: &bind_group_entries,
                    label: None,
                });

                self.pose = pose;

                self.buffers = Some(
                    Buffers {
                        bind_group,
                        transform,
                    }
                )
            }
        }
    }

    pub(crate) fn draw(
        &self,
        assets: &Assets,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &Pipeline,
        frame: &wgpu::SwapChainTexture,
        depth_buffer: &wgpu::TextureView,
    ) {
        if let Some(buffers) = self.buffers.as_ref() {
            let mesh = assets
                .get(self.mesh)
                .expect("Static model must have a mesh");

            let vertices_buffer = mesh
                .vertices_buffer
                .as_ref()
                .expect("Static model mesh must have initialized buffers at this stage");

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
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachmentDescriptor {
                        attachment: depth_buffer,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }),
                        stencil_ops: None,
                    }
                ),
            });

            rpass.push_debug_group("Prepare to draw a Model");
            rpass.set_pipeline(&pipeline.wgpu_pipeline);
            rpass.set_bind_group(0, &buffers.bind_group, &[]);
            rpass.set_vertex_buffer(0, vertices_buffer.slice(..));
            rpass.pop_debug_group();

            if let Some(indices_buffer) = mesh.indices_buffer.as_ref() {
                rpass.insert_debug_marker("Draw indexed model");
                rpass.set_index_buffer(indices_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..mesh.indices_count(), 0, 0..1);
            } else {
                rpass.insert_debug_marker("Draw a model");
                rpass.draw(0..mesh.indices_count(), 0..1);
            }
        }
    }
}
