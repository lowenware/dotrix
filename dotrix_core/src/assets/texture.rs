
/// Texture asset
#[derive(Default)]
pub struct Texture {
    /// Texture width in pixels
    pub width: u32,
    /// Texture height in pixels
    pub height: u32,
    /// Texture depth
    pub depth: u32,
    /// Raw texture data
    pub data: Vec<u8>,
    /// Texture buffer
    pub view: Option<wgpu::TextureView>,
}

impl Texture {
    /// Loads the [`Texture`] data to a buffer
    pub fn load(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.view.is_some() {
            return;
        }

        let texture_extent = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
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

        let bytes_per_row = std::num::NonZeroU32::new(self.data.len() as u32 / self.height)
            .unwrap();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &self.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(std::num::NonZeroU32::new(self.height).unwrap()),
            },
            texture_extent,
        );
    }

    /// Unloads the [`Texture`] data from the buffer
    pub fn unload(&mut self) {
        self.view.take();
    }

    /// Returns a view of the [`Texture`]
    pub fn view(&self) -> &wgpu::TextureView {
        self.view.as_ref().unwrap()
    }
}
