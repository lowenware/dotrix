//! Component and buffers

use crate::{
    assets::{ Id, Texture, Mesh },
    services::Assets,
};

use super::pipeline::Pipeline;
use dotrix_math::Mat4;

/// Pipeline buffers
pub struct Buffers {
    bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    proj_view: wgpu::Buffer,
    indices_count: u32,
}

/// SkyBox component
///
/// SkyBox is a cube with 6 textures on internal sides. It has one major difference from the rgular
/// cube though: SkyBox is fixed relatively to camera position.
///
/// Usage is quite straight forward. You need 6 textures and spawn an entity with the compomnet.
/// 
/// ```no_run
/// use dotrix_core::{
///     assets::Texture,
///     components::SkyBox,
///     ecs::Mut,
///     services::{ Assets, World },
/// };
///
/// fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
///     let primary_texture = [
///         assets.register::<Texture>("skybox_right"),
///         assets.register::<Texture>("skybox_left"),
///         assets.register::<Texture>("skybox_top"),
///         assets.register::<Texture>("skybox_bottom"),
///         assets.register::<Texture>("skybox_back"),
///         assets.register::<Texture>("skybox_front"),
///     ];
///
///     assets.import("examples/skybox/skybox_right.png");
///     assets.import("examples/skybox/skybox_left.png");
///     assets.import("examples/skybox/skybox_top.png");
///     assets.import("examples/skybox/skybox_bottom.png");
///     assets.import("examples/skybox/skybox_front.png");
///     assets.import("examples/skybox/skybox_back.png");
///
///     world.spawn(vec![
///         (SkyBox { primary_texture, ..Default::default() },),
///     ]);
/// }
/// ```
/// Dotrix provides a simple
/// [example](https://github.com/lowenware/dotrix/blob/main/examples/skybox/skybox.rs) of how to
/// use the [`SkyBox`].
#[derive(Default)]
pub struct SkyBox {
    /// Slice of [`Id`] of [`Texture`] assets
    pub primary_texture: [Id<Texture>; 6],
    /// Second texture is for the smooth change (not implemented yet)
    pub secondary_texture: Option<[Id<Texture>; 6]>,
    /// Pipline buffers
    pub buffers: Option<Buffers>,
    /// [`Id`] of a rendering [`Pipeline`]
    pub pipeline: Id<Pipeline>,
}

impl SkyBox {

    fn faces<'a>(&self, assets: &'a Assets) -> Option<Vec<&'a Texture>> {
        let mut faces = Vec::new();
        for texture_id in self.primary_texture.iter() {
            if let Some(face) = assets.get(*texture_id) {
                faces.push(face);
            } else {
                return None;
            }
        }
        Some(faces)
    }

    /// Loads the [`SkyBox`] buffers
    pub fn load(
        &mut self,
        assets: &Assets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pipeline: &Pipeline,
        sampler_3d: &wgpu::Sampler,
        proj_view_matrix: &Mat4,
    ) {
        use wgpu::util::DeviceExt;

        let proj_view_slice = AsRef::<[f32; 16]>::as_ref(proj_view_matrix);
        if let Some(buffers) = self.buffers.as_ref() {
            queue.write_buffer(&buffers.proj_view, 0, bytemuck::cast_slice(proj_view_slice));
        } else {
            self.buffers = if let Some(faces) = self.faces(assets) {

                let cube = Mesh::cube();

                let extent = wgpu::Extent3d {
                    width: faces[0].width,
                    height: faces[0].height,
                    depth_or_array_layers: 6,
                };

                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: extent,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
                    label: None,
                });

                for (i, image) in faces.iter().enumerate() {
                    queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: i as u32,
                            },
                        },
                        &image.data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(std::num::NonZeroU32::new(4 * image.width).unwrap()),
                            rows_per_image: None,
                        },
                        wgpu::Extent3d {
                            width: faces[i].width,
                            height: faces[i].height,
                            depth_or_array_layers: 1,
                        },
                    );
                }

                let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                    label: None,
                    dimension: Some(wgpu::TextureViewDimension::Cube),
                    ..wgpu::TextureViewDescriptor::default()
                });

                let proj_view = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ProjView (static) Buffer"),
                    contents: bytemuck::cast_slice(proj_view_slice),
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                });

                let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&cube.positions),
                    usage: wgpu::BufferUsage::VERTEX,
                });

                let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(cube.indices.as_ref().unwrap()),
                    usage: wgpu::BufferUsage::INDEX,
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
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(sampler_3d),
                        },
                    ],
                    label: None,
                });

                Some(Buffers {
                    bind_group,
                    proj_view,
                    vertices,
                    indices,
                    indices_count: cube.indices_count(),
                })
            } else {
                None
            };
        }
    }

    /// Renders the [`SkyBox`]
    pub(crate) fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &Pipeline,
        frame: &wgpu::SwapChainTexture,
    ) {
        if let Some(buffers) = self.buffers.as_ref() {

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true, 
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.push_debug_group("Prepare SkyBox for draw");
            rpass.set_pipeline(&pipeline.wgpu_pipeline);
            rpass.set_bind_group(0, &buffers.bind_group, &[]);
            rpass.set_vertex_buffer(0, buffers.vertices.slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("draw SkyBox");
            rpass.set_index_buffer(buffers.indices.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..buffers.indices_count, 0, 0..1);
        }
    }
}
