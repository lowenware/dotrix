
#[derive(Default)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub data: Vec<u8>,
    pub view: Option<wgpu::TextureView>,
}

impl Texture {
    pub fn load(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.view.is_some() {
            return;
        }

        let texture_extent = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        self.view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &self.data,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * self.width,
                rows_per_image: 0,
            },
            texture_extent,
        );
    }

    pub fn unload(&mut self) {
        self.view.take();
    }

    pub fn view(&self) -> &wgpu::TextureView {
        self.view.as_ref().unwrap()
    }
}
