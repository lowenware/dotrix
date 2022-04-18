use super::{Buffer, Context};
use wgpu;

pub enum TextureKind {
    D2,
    Cube,
    D2Array,
    D3,
}

/// GPU Texture Implementation
pub struct Texture {
    /// Texture label
    pub label: String,
    /// WGPU Texture view
    pub wgpu_texture_view: Option<wgpu::TextureView>,
    /// WGPU Texture
    pub wgpu_texture: Option<wgpu::Texture>,
    /// Texture usage
    pub usage: wgpu::TextureUsages,
    /// Texture kind
    pub kind: TextureKind,
    /// Texture format
    pub format: wgpu::TextureFormat,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            label: String::from("Noname Texture"),
            wgpu_texture_view: None,
            wgpu_texture: None,
            usage: wgpu::TextureUsages::empty(),
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            kind: TextureKind::D2,
        }
    }
}

impl Texture {
    /// Constructs GPU Texture
    pub fn new(label: &str) -> Self {
        Self {
            label: String::from(label),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            ..Default::default()
        }
    }

    /// Constructs a CubeMap GPU Texture
    pub fn new_cube(label: &str) -> Self {
        Self {
            label: String::from(label),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            kind: TextureKind::Cube,
            ..Default::default()
        }
    }

    /// Constructs a 2D Array GPU Texture
    pub fn new_array(label: &str) -> Self {
        Self {
            label: String::from(label),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            kind: TextureKind::D2Array,
            ..Default::default()
        }
    }

    /// Constructs a 3D GPU Texture
    pub fn new_3d(label: &str) -> Self {
        Self {
            label: String::from(label),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            kind: TextureKind::D3,
            ..Default::default()
        }
    }

    /// Constructs GPU Storage Texture
    pub fn storage(label: &str) -> Self {
        Self {
            label: String::from(label),
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            ..Default::default()
        }
    }

    /// Constructs GPU Storage Texture
    pub fn attachment(label: &str) -> Self {
        Self {
            label: String::from(label),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            ..Default::default()
        }
    }

    /// Set Texture format to Rgba8UnormSrgb
    #[must_use]
    pub fn rgba_u8norm_srgb(mut self) -> Self {
        self.format = wgpu::TextureFormat::Rgba8UnormSrgb;
        self
    }

    /// Set Texture format to Depth32Float
    #[must_use]
    pub fn depth_f32(mut self) -> Self {
        self.format = wgpu::TextureFormat::Depth32Float;
        self
    }

    /// Allow to use as Texture
    #[must_use]
    pub fn use_as_texture(mut self) -> Self {
        self.usage |= wgpu::TextureUsages::TEXTURE_BINDING;
        self
    }

    /// Allow to use as Storage
    #[must_use]
    pub fn use_as_storage(mut self) -> Self {
        self.usage |= wgpu::TextureUsages::STORAGE_BINDING;
        self
    }

    /// Allow to use as Attachment
    #[must_use]
    pub fn use_as_attachment(mut self) -> Self {
        self.usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        self
    }

    /// Allow reading from buffer
    #[must_use]
    pub fn allow_read(mut self) -> Self {
        self.usage |= wgpu::TextureUsages::COPY_DST;
        self
    }

    /// Allow writing to buffer
    #[must_use]
    pub fn allow_write(mut self) -> Self {
        self.usage |= wgpu::TextureUsages::COPY_SRC;
        self
    }

    /// Loads data into the texture buffer
    pub(crate) fn load<'a>(&mut self, ctx: &Context, width: u32, height: u32, layers: &[&'a [u8]]) {
        let dimension = match self.kind {
            TextureKind::D2 => wgpu::TextureViewDimension::D2,
            TextureKind::Cube => {
                assert!(layers.len() == 6);
                wgpu::TextureViewDimension::Cube
            }
            TextureKind::D2Array => wgpu::TextureViewDimension::D2Array,
            TextureKind::D3 => wgpu::TextureViewDimension::D3,
        };

        let format = self.format;
        let usage = self.usage;
        let depth_or_array_layers = layers.len() as u32;
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };
        let layer_size = wgpu::Extent3d {
            depth_or_array_layers: 1,
            ..size
        };
        let max_mips = 1;

        let tex_dimension: wgpu::TextureDimension = match self.kind {
            TextureKind::D2 => wgpu::TextureDimension::D2,
            TextureKind::Cube => wgpu::TextureDimension::D2,
            TextureKind::D2Array => wgpu::TextureDimension::D2,
            TextureKind::D3 => wgpu::TextureDimension::D3,
        };

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&self.label),
            size,
            mip_level_count: max_mips as u32,
            sample_count: 1,
            dimension: tex_dimension,
            format,
            usage,
        });

        self.wgpu_texture_view = Some(texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(format),
            dimension: Some(dimension),
            ..wgpu::TextureViewDescriptor::default()
        }));

        for (i, data) in layers.iter().enumerate() {
            let bytes_per_row = std::num::NonZeroU32::new(data.len() as u32 / height).unwrap();
            ctx.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(std::num::NonZeroU32::new(height).unwrap()),
                },
                layer_size,
            );
        }

        self.wgpu_texture = Some(texture);
    }

    /// Checks if texture is loaded
    pub fn loaded(&self) -> bool {
        self.wgpu_texture_view.is_some()
    }

    /// Release all resources used by the texture
    pub fn unload(&mut self) {
        self.wgpu_texture_view.take();
    }

    /// Get unwrapped reference to WGPU Texture View
    pub fn get(&self) -> &wgpu::TextureView {
        self.wgpu_texture_view
            .as_ref()
            .expect("Texture must be loaded")
    }

    /// Check if the texture format is filterable
    pub fn is_filterable(&self) -> bool {
        self.format.describe().guaranteed_format_features.filterable
    }

    /// Get the texture bytes per pixels
    pub fn pixel_bytes(&self) -> u8 {
        self.format.describe().block_size
    }

    /// Get the number of channels
    pub fn num_channels(&self) -> u8 {
        self.format.describe().components
    }

    /// Get the texture sample type (float/uint etc)
    pub fn sample_type(&self) -> wgpu::TextureSampleType {
        self.format.describe().sample_type
    }

    /// Fetch data from the gpu
    ///
    /// This is useful textures that are altered on the gpu
    ///
    /// This operation is slow and should mostly be
    /// used for debugging
    pub fn fetch_from_gpu(
        &self,
        dimensions: [u32; 3],
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, wgpu::BufferAsyncError>> {
        let bytes_per_pixel: u32 = self.pixel_bytes() as u32;
        let mut staging_buffer = Buffer::map_read("Texture Fetch Staging buffer");
        let unpadded_bytes_per_row: u32 =
            std::num::NonZeroU32::new(bytes_per_pixel as u32 * dimensions[0])
                .unwrap()
                .into();
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;

        staging_buffer.create(
            ctx,
            padded_bytes_per_row * dimensions[0] * dimensions[1],
            false,
        );
        ctx.run_copy_texture_to_buffer(self, &staging_buffer, dimensions, bytes_per_pixel);

        async move {
            // TODO: Urgently work out a better way to await the next frame.
            std::thread::sleep(std::time::Duration::from_secs(1));

            let wgpu_buffer = staging_buffer.wgpu_buffer.expect("Buffer must be loaded");
            let buffer_slice = wgpu_buffer.slice(..);
            // Gets the future representing when `staging_buffer` can be read from
            let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

            match buffer_future.await {
                Ok(()) => {
                    // Gets contents of buffer
                    let data = buffer_slice.get_mapped_range();
                    // This strips the padding on each row
                    let result: Vec<u8> = data
                        .chunks_exact((padded_bytes_per_row * dimensions[1]) as usize)
                        .flat_map(|img| {
                            let rows: Vec<Vec<u8>> = img
                                .chunks_exact(padded_bytes_per_row as usize)
                                .map(|row| row[0..(unpadded_bytes_per_row as usize)].to_vec())
                                .collect();
                            rows
                        })
                        .flatten()
                        .collect();

                    // With the current interface, we have to make sure all mapped views are
                    // dropped before we unmap the buffer.
                    drop(data);
                    wgpu_buffer.unmap(); // Unmaps buffer from memory
                                         // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                         //   delete myPointer;
                                         //   myPointer = NULL;
                                         // It effectively frees the memory
                                         //
                    Ok(result)
                }

                Err(e) => Err(e),
            }
        }
    }
}
