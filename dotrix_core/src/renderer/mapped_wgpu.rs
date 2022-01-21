//! Holds various structure mappings between dotrix and wgpu
//!
//! These are done so that we can limit how much source must be
//! rewritten if wgpu changes any names.
//!
pub use wgpu::TextureFormat as WgpuTextureFormat;

/// Defines the intended texture usages
///
/// Should be built with `create()`
/// ```norun
/// TextureUsages::create().read().write().render()
/// ```
#[derive(Copy, Clone, Debug)]
pub struct TextureUsages {
    read: bool,
    write: bool,
    as_texture: bool,
    as_storage: bool,
    as_render: bool,
}

impl Default for TextureUsages {
    fn default() -> Self {
        Self::create().write().texture()
    }
}

impl TextureUsages {
    #[must_use]
    /// Create a texture with all usages
    pub fn all() -> Self {
        Self {
            read: true,
            write: true,
            as_texture: true,
            as_storage: true,
            as_render: true,
        }
    }
    /// Create a texture with no usage
    #[must_use]
    pub fn none() -> Self {
        Self {
            read: false,
            write: false,
            as_texture: false,
            as_storage: false,
            as_render: false,
        }
    }
    /// Start creating a texture usage desciption
    #[must_use]
    pub fn create() -> Self {
        Self::none()
    }
    /// Allow the texture's gpu buffer to be read from
    #[must_use]
    pub fn read(mut self) -> Self {
        self.read = true;
        self
    }
    /// Allow the texture's gpu buffer to be written to
    #[must_use]
    pub fn write(mut self) -> Self {
        self.write = true;
        self
    }
    /// Allow the texture's gpu buffer to be used as a render attachment
    #[must_use]
    pub fn render(mut self) -> Self {
        self.as_render = true;
        self
    }
    /// Allow the texture's gpu buffer to be bound as a StorageTexture
    #[must_use]
    pub fn storage(mut self) -> Self {
        self.as_storage = true;
        self
    }
    /// Allow the texture's gpu buffer to be bound as a Texture
    #[must_use]
    pub fn texture(mut self) -> Self {
        self.as_texture = true;
        self
    }
}

impl From<TextureUsages> for wgpu::TextureUsages {
    fn from(obj: TextureUsages) -> wgpu::TextureUsages {
        let mut res = wgpu::TextureUsages::empty();
        if obj.read {
            res |= wgpu::TextureUsages::COPY_SRC
        }
        if obj.write {
            res |= wgpu::TextureUsages::COPY_DST
        }
        if obj.as_render {
            res |= wgpu::TextureUsages::RENDER_ATTACHMENT
        }
        if obj.as_storage {
            res |= wgpu::TextureUsages::STORAGE_BINDING
        }
        if obj.as_texture {
            res |= wgpu::TextureUsages::TEXTURE_BINDING
        }
        res
    }
}

/// Defines the possible access modes for a
/// storage texture.
#[derive(Copy, Clone, Debug)]
pub enum StorageTextureAccess {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Read write access
    ReadWrite,
}

impl From<StorageTextureAccess> for wgpu::StorageTextureAccess {
    fn from(obj: StorageTextureAccess) -> Self {
        match obj {
            StorageTextureAccess::Read => Self::ReadOnly,
            StorageTextureAccess::Write => Self::WriteOnly,
            StorageTextureAccess::ReadWrite => Self::ReadWrite,
        }
    }
}

/// The texture format used while on the gpu
#[derive(Copy, Clone, Debug)]
pub struct TextureFormat {
    /// The raw `[wgpu::TextureFormat]`
    pub wgpu_texture_format: WgpuTextureFormat,
}

impl TextureFormat {
    /// Red channel unsigned 8 bit integer
    pub fn r_u8() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R8Uint,
        }
    }
    /// Red channel signed 8 bit integer
    pub fn r_i8() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R8Sint,
        }
    }
    /// Red channel unsigned 8 bit integer normalised to 0..1 in the shader
    pub fn r_u8norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R8Unorm,
        }
    }
    /// Red channel signed 8 bit integer normalised to -1..1 in the shader
    pub fn r_i8norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R8Snorm,
        }
    }

    /// Red channel unsigned 16 bit integer
    pub fn r_u16() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R16Uint,
        }
    }
    /// Red channel signed 16 bit integer
    pub fn r_i16() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R16Sint,
        }
    }
    /// Red channel unsigned 16 bit integer normalised to 0..1 in the shader
    pub fn r_u16norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R16Unorm,
        }
    }
    /// Red channel signed 16 bit integer normalised to -1..1 in the shader
    pub fn r_i16norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R16Snorm,
        }
    }
    /// Red channel unsigned 16 bit float
    pub fn r_f16() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R16Float,
        }
    }
    /// Red channel unsigned 32 bit integer
    pub fn r_u32() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R32Uint,
        }
    }
    /// Red channel signed 32 bit integer
    pub fn r_i32() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R32Sint,
        }
    }
    /// Red channel 32 bit float
    pub fn r_f32() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::R32Float,
        }
    }

    // RG channels
    /// Red and green channel unsigned 8 bit integer
    pub fn rg_u8() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg8Uint,
        }
    }
    /// Red and green channel signed 8 bit integer
    pub fn rg_i8() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg8Sint,
        }
    }
    /// Red and green channel unsigned 8 bit integer normalised to 0..1 in the shader
    pub fn rg_u8norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg8Unorm,
        }
    }
    /// Red and green channel signed 8 bit integer normalised to -1..1 in the shader
    pub fn rg_i8norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg8Snorm,
        }
    }

    /// Red and green channel unsigned 16 bit integer
    pub fn rg_u16() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg16Uint,
        }
    }
    /// Red and green channel signed 16 bit integer
    pub fn rg_i16() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg16Sint,
        }
    }
    /// Red and green channel unsigned 16 bit integer normalised to 0..1 in the shader
    pub fn rg_u16norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg16Unorm,
        }
    }
    /// Red and green channel signed 16 bit integer normalised to -1..1 in the shader
    pub fn rg_i16norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg16Snorm,
        }
    }
    /// Red and green channel unsigned 16 bit float
    pub fn rg_f16() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg16Float,
        }
    }
    /// Red and green channel unsigned 32 bit integer
    pub fn rg_u32() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg32Uint,
        }
    }
    /// Red and green channel signed 32 bit integer
    pub fn rg_i32() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg32Sint,
        }
    }
    /// Red and green channel 32 bit float
    pub fn rg_f32() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rg32Float,
        }
    }

    // RGBA Channels
    /// Red, green, blue and aplha channel unsigned 8 bit integer
    pub fn rgba_u8() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba8Uint,
        }
    }
    /// Red, green, blue and aplha channel signed 8 bit integer
    pub fn rgba_i8() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba8Sint,
        }
    }
    /// Red, green, blue and aplha channel unsigned 8 bit integer normalised to 0..1 in the shader
    pub fn rgba_u8norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba8Unorm,
        }
    }
    /// Red, green, blue and aplha channel unsigned 8 bit integer normalised to 0..1 in the shader
    /// with sRGB color space
    pub fn rgba_u8norm_srgb() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba8UnormSrgb,
        }
    }
    /// Red, green, blue and aplha channel signed 8 bit integer normalised to -1..1 in the shader
    pub fn rgba_i8norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba8Snorm,
        }
    }

    /// Red, green, blue and aplha channel unsigned 16 bit integer
    pub fn rgba_u16() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba16Uint,
        }
    }
    /// Red, green, blue and aplha channel signed 16 bit integer
    pub fn rgba_i16() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba16Sint,
        }
    }
    /// Red, green, blue and aplha channel unsigned 16 bit integer normalised to 0..1 in the shader
    pub fn rgba_u16norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba16Unorm,
        }
    }
    /// Red, green, blue and aplha channel signed 16 bit integer normalised to -1..1 in the shader
    pub fn rgba_i16norm() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba16Snorm,
        }
    }
    /// Red, green, blue and aplha channel unsigned 16 bit float
    pub fn rgba_f16() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba16Float,
        }
    }
    /// Red, green, blue and aplha channel unsigned 32 bit integer
    pub fn rgba_u32() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba32Uint,
        }
    }
    /// Red, green, blue and aplha channel signed 32 bit integer
    pub fn rgba_i32() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba32Sint,
        }
    }
    /// Red, green, blue and aplha channel 32 bit float
    pub fn rgba_f32() -> Self {
        Self {
            wgpu_texture_format: WgpuTextureFormat::Rgba32Float,
        }
    }

    pub(crate) fn is_filterable(&self) -> bool {
        self.wgpu_texture_format
            .describe()
            .guaranteed_format_features
            .filterable
    }
}

impl From<TextureFormat> for WgpuTextureFormat {
    fn from(orig: TextureFormat) -> Self {
        orig.wgpu_texture_format
    }
}
