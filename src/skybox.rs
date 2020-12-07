pub use crate::{
    assets::{Id, Texture},
    services::Assets,
    renderer::skybox::RendererContext,
};

pub struct Buffers {
    pub bind_group: wgpu::BindGroup,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub indices_count: u32,
}

#[derive(Default)]
pub struct SkyBox {
    pub primary_texture: [Id<Texture>; 6],
    pub secondary_texture: Option<[Id<Texture>; 6]>,
    pub buffers: Option<Buffers>,
}

impl SkyBox {

    fn vertices() -> Vec<[f32; 3]> {
        vec![
            // front
            [-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0],
            // top 
            [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0], [-1.0, 1.0, 1.0], [1.0, 1.0, 1.0],
            // right
            [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [1.0, 1.0, 1.0], [1.0, -1.0, 1.0],
            // left
            [-1.0, -1.0, 1.0], [-1.0, 1.0, 1.0], [-1.0, 1.0, -1.0], [-1.0, -1.0, -1.0],
            // back
            [-1.0, 1.0, -1.0], [1.0, 1.0, -1.0], [1.0, -1.0, -1.0], [-1.0, -1.0, -1.0],
            // bottom
            [1.0, -1.0, 1.0], [-1.0, -1.0, 1.0], [-1.0, -1.0, -1.0], [1.0, -1.0, -1.0],
        ]
    }

    fn indices() -> Vec<u16> {
        vec![
            0, 1, 2, 2, 3, 0,
            4, 5, 6, 6, 7, 4,
            8, 9, 10, 10, 11, 8,
            12, 13, 14, 14, 15, 12,
            16, 17, 18, 18, 19, 16,
            20, 21, 22, 22, 23, 20,
        ]
    }

    pub fn faces<'a>(&self, assets: &'a Assets) -> Option<Vec<&'a Texture>> {
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

    pub fn try_init_buffers(
        &mut self,
        assets: &Assets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ctx: &RendererContext,
    ) {
        use wgpu::util::DeviceExt;

        self.buffers = if let Some(faces) = self.faces(assets) {

            let extent = wgpu::Extent3d {
                width: faces[0].width,
                height: faces[0].height,
                depth: 6,
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
                    wgpu::TextureCopyView {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: i as u32,
                        },
                    },
                    &image.data,
                    wgpu::TextureDataLayout {
                        offset: 0,
                        bytes_per_row: 4 * image.width,
                        rows_per_image: 0,
                    },
                    wgpu::Extent3d {
                        width: faces[i].width,
                        height: faces[i].height,
                        depth: 1,
                    },
                );
            }

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..wgpu::TextureViewDescriptor::default()
            });

            let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&Self::vertices()),
                usage: wgpu::BufferUsage::VERTEX,
            });

            let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&Self::indices()),
                usage: wgpu::BufferUsage::INDEX,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &ctx.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: ctx.proj_view.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                    },
                ],
                label: None,
            });

            Some(Buffers {
                bind_group,
                vertices,
                indices,
                indices_count: 36,
            })
        } else {
            None
        };
    }
}
