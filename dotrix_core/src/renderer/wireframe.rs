use crate::{
    assets::{ Id, Wires },
    services::{ Assets, Renderer },
};

use super::pipeline::Pipeline;
use super::transform::Transform;

/// Pipeline buffers
pub struct Buffers {
    bind_group: wgpu::BindGroup,
    transform: wgpu::Buffer,
}

/// Component to draw wire frames
///
/// Typical use of the component is to render a wired cube
///
/// ```
/// use dotrix_core::{
///     assets::Wires,
///     components::WireFrame,
///     ecs::Mut,
///     services::{ Assets, World },
/// };
///
/// fn my_system( mut assets: Mut<Assets>, mut world: Mut<World>) {
///     let wires = assets.store(Wires::cube([1.0; 3]));
///     world.spawn(
///         Some((
///             WireFrame { wires, ..Default::default() },
///         ))
///     );
/// }
/// ```
#[derive(Default)]
pub struct WireFrame {
    /// wires asset id
    pub wires: Id<Wires>,
    /// transformations
    pub transform: Transform,
    /// custom [`Id`] of a rendering [`Pipeline`]
    pub pipeline: Id<Pipeline>,
    /// is rendering disabled
    pub disabled: bool,
    /// pipeline buffers
    pub buffers: Option<Buffers>,
}

impl WireFrame {

    /// Returns loaded assets if they are all ready
    fn get_assets<'a>(
        &self,
        assets: &'a mut Assets,
        device: &wgpu::Device,
    ) -> Result<&'a Wires, ()> {

        if let Some(wires) = assets.get_mut(self.wires) {
            wires.load(device);
        }

        if let Some(wires) = assets.get(self.wires) {
            return Ok(wires);
        }
        Err(())
    }

    /// Initialize model specific buffers, should be called just once by renderer
    pub(crate) fn load(
        &mut self,
        renderer: &Renderer,
        assets: &mut Assets,
        pipeline: &Pipeline,
        proj_view: &wgpu::Buffer,
    ) {
        use wgpu::util::DeviceExt;

        let device = &renderer.device;
        let queue = &renderer.queue;

        let transform_matrix = self.transform.matrix();
        let wire_frame_transform = AsRef::<[f32; 16]>::as_ref(&transform_matrix);

        if self.get_assets(assets, device).is_ok() {

            if let Some(buffers) = self.buffers.as_ref() {
                queue.write_buffer(
                    &buffers.transform,
                    0,
                    bytemuck::cast_slice(wire_frame_transform)
                );
            } else {
                let transform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("WireFrame Transform"),
                    contents: bytemuck::cast_slice(wire_frame_transform),
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &pipeline.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: proj_view.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: transform.as_entire_binding(),
                        },
                    ],
                    label: None,
                });

                self.buffers = Some(
                    Buffers {
                        bind_group,
                        transform,
                    }
                );
            }
        }
    }

    /// draw the [`WireFrame`]
    pub(crate) fn draw(
        &self,
        assets: &Assets,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &Pipeline,
        frame: &wgpu::SwapChainTexture,
        depth_buffer: &wgpu::TextureView,
    ) {
        if let Some(buffers) = self.buffers.as_ref() {
            let wires = assets
                .get(self.wires)
                .expect("WireFrame must have wires");

            let vertices_buffer = wires
                .vertices_buffer
                .as_ref()
                .expect("WireFrame must have initialized buffers at this stage");

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, 
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachment {
                        view: depth_buffer,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }),
                        stencil_ops: None,
                    }
                ),
            });

            rpass.push_debug_group("Prepare to draw a WireFrame");
            rpass.set_pipeline(&pipeline.wgpu_pipeline);
            rpass.set_bind_group(0, &buffers.bind_group, &[]);
            rpass.set_vertex_buffer(0, vertices_buffer.slice(..));
            rpass.pop_debug_group();

            rpass.insert_debug_marker("Draw a WireFrame");
            rpass.draw(0..wires.vertices_count(), 0..1);
        }
    }
}
