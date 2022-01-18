//! Holds various structure mappings between dotrix and wgpu
//!
//! These are done so that we can limit how much source must be
//! rewritten if wgpu changes any names.
//!

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::TryFrom;
use thiserror::Error;

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

/// Error that can occur when no applicable wgpu texture format can be
/// found
#[derive(Error, Debug)]
pub enum WgpuConversionError {
    #[error("No combination of input patameters can be represented in wgpu")]
    InvalidCombination,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum DataSize {
    Bits8,
    Bits16,
    Bits32,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum DataFormat {
    Unorm,
    Snorm,
    Uint,
    Sint,
    Float,
    UnormSrgb,
}

/// Defines the texture format
///
/// Shold be built using
/// ```norun
/// TextureFormat::create().r().g().b().a().bits8().snorm()
/// ```
#[derive(Copy, Clone, Debug)]
pub struct TextureFormat {
    r: bool,
    g: bool,
    b: bool,
    a: bool,
    size: DataSize,
    format: DataFormat,
}
impl TextureFormat {
    /// Start creating a texture format
    #[must_use]
    pub fn create() -> Self {
        Self {
            r: false,
            g: false,
            b: false,
            a: false,
            size: DataSize::Bits8,
            format: DataFormat::Snorm,
        }
    }
    /// Include the red channel
    #[must_use]
    pub fn r(mut self) -> Self {
        self.r = true;
        self
    }
    /// Include the green channel
    #[must_use]
    pub fn g(mut self) -> Self {
        self.g = true;
        self
    }
    /// Include the blue channel
    #[must_use]
    pub fn b(mut self) -> Self {
        self.b = true;
        self
    }
    /// Include the alpha channel
    #[must_use]
    pub fn a(mut self) -> Self {
        self.a = true;
        self
    }
    /// Use 8 bit channels (this will supercede any prior size)
    #[must_use]
    pub fn bits8(mut self) -> Self {
        self.size = DataSize::Bits8;
        self
    }
    /// Use 16 bit channels (this will supercede any prior size)
    #[must_use]
    pub fn bits16(mut self) -> Self {
        self.size = DataSize::Bits16;
        self
    }
    /// Use 32 bit channels (this will supercede any prior size)
    #[must_use]
    pub fn bits32(mut self) -> Self {
        self.size = DataSize::Bits32;
        self
    }
    /// Use unsigned normalised format
    #[must_use]
    pub fn unorm(mut self) -> Self {
        self.format = DataFormat::Unorm;
        self
    }
    /// Use signed normalised format
    #[must_use]
    pub fn snorm(mut self) -> Self {
        self.format = DataFormat::Snorm;
        self
    }
    /// Use unsigned integer format
    #[must_use]
    pub fn uint(mut self) -> Self {
        self.format = DataFormat::Uint;
        self
    }
    /// Use signed integer format
    #[must_use]
    pub fn sint(mut self) -> Self {
        self.format = DataFormat::Sint;
        self
    }
    /// Use floating point format
    #[must_use]
    pub fn float(mut self) -> Self {
        self.format = DataFormat::Float;
        self
    }
    /// Use signed normalised format with srgb color space
    #[must_use]
    pub fn unorm_srgb(mut self) -> Self {
        self.format = DataFormat::UnormSrgb;
        self
    }
}

lazy_static! {
    // Hashmap order is, channels => size => format
    static ref TEX_FORMAT_MAP: HashMap<u8, HashMap<DataSize, HashMap<DataFormat, wgpu::TextureFormat>>> = {
        HashMap::from([
            // 1 Channel (Red)
            (1,
                HashMap::from([
                    (DataSize::Bits8,
                        HashMap::from([
                            (DataFormat::Unorm,wgpu::TextureFormat::R8Unorm),
                            (DataFormat::Snorm,wgpu::TextureFormat::R8Snorm),
                            (DataFormat::Uint,wgpu::TextureFormat::R8Uint),
                            (DataFormat::Sint,wgpu::TextureFormat::R8Sint),
                        ])
                    ),
                    (DataSize::Bits16,
                        HashMap::from([
                            (DataFormat::Unorm,wgpu::TextureFormat::R16Unorm),
                            (DataFormat::Snorm,wgpu::TextureFormat::R16Snorm),
                            (DataFormat::Uint,wgpu::TextureFormat::R16Uint),
                            (DataFormat::Sint,wgpu::TextureFormat::R16Sint),
                            (DataFormat::Float,wgpu::TextureFormat::R16Float),
                        ])
                    ),
                    (DataSize::Bits32,
                        HashMap::from([
                            (DataFormat::Uint,wgpu::TextureFormat::R32Uint),
                            (DataFormat::Sint,wgpu::TextureFormat::R32Sint),
                            (DataFormat::Float,wgpu::TextureFormat::R32Float),
                        ])
                    )
                ]),
            ),
            // 2 Channel (Red, Green)
            (2,
                HashMap::from([
                    (DataSize::Bits8,
                        HashMap::from([
                            (DataFormat::Unorm,wgpu::TextureFormat::Rg16Unorm),
                            (DataFormat::Snorm,wgpu::TextureFormat::Rg16Snorm),
                            (DataFormat::Uint,wgpu::TextureFormat::Rg16Uint),
                            (DataFormat::Sint,wgpu::TextureFormat::Rg16Sint),
                            (DataFormat::Float,wgpu::TextureFormat::Rg16Float),
                        ])
                    ),
                    (DataSize::Bits16,
                        HashMap::from([
                            (DataFormat::Unorm,wgpu::TextureFormat::Rg8Unorm),
                            (DataFormat::Snorm,wgpu::TextureFormat::Rg8Snorm),
                            (DataFormat::Uint,wgpu::TextureFormat::Rg8Uint),
                            (DataFormat::Sint,wgpu::TextureFormat::Rg8Sint),
                        ])
                    ),
                    (DataSize::Bits32,
                        HashMap::from([
                            (DataFormat::Uint,wgpu::TextureFormat::Rg32Uint),
                            (DataFormat::Sint,wgpu::TextureFormat::Rg32Sint),
                            (DataFormat::Float,wgpu::TextureFormat::Rg32Float),
                        ])
                    ),
                ]),
            ),
            // 3 Channel (Red, Green, Blue)
            (3, Default::default()),
            // 4 Channel (Red, Green, Blue, Alpha)
            (4,
                HashMap::from([
                    (DataSize::Bits8,
                        HashMap::from([
                            (DataFormat::Unorm,wgpu::TextureFormat::Rgba8Unorm),
                            (DataFormat::Snorm,wgpu::TextureFormat::Rgba8Snorm),
                            (DataFormat::Uint,wgpu::TextureFormat::Rgba8Uint),
                            (DataFormat::Sint,wgpu::TextureFormat::Rgba8Sint),
                            (DataFormat::UnormSrgb,wgpu::TextureFormat::Rgba8UnormSrgb),
                        ]),
                    ),
                    (DataSize::Bits16,
                        HashMap::from([
                            (DataFormat::Unorm,wgpu::TextureFormat::Rgba16Unorm),
                            (DataFormat::Snorm,wgpu::TextureFormat::Rgba16Snorm),
                            (DataFormat::Uint,wgpu::TextureFormat::Rgba16Uint),
                            (DataFormat::Sint,wgpu::TextureFormat::Rgba16Sint),
                            (DataFormat::Float,wgpu::TextureFormat::Rgba16Float),
                        ])
                    ),
                    (DataSize::Bits32,
                        HashMap::from([
                            (DataFormat::Uint,wgpu::TextureFormat::Rgba32Uint),
                            (DataFormat::Sint,wgpu::TextureFormat::Rgba32Sint),
                            (DataFormat::Float,wgpu::TextureFormat::Rgba32Float),
                        ])
                    ),
                ]),
            ),
        ])
    };
}

impl TryFrom<TextureFormat> for wgpu::TextureFormat {
    type Error = WgpuConversionError;
    fn try_from(value: TextureFormat) -> Result<Self, Self::Error> {
        let channels = [value.r, value.g, value.b, value.a]
            .iter()
            .filter(|&&v| v)
            .count();
        if let Some(Some(Some(&format))) = TEX_FORMAT_MAP.get(&(channels as u8)).map(|size_map| {
            size_map
                .get(&value.size)
                .map(|format_map| format_map.get(&value.format))
        }) {
            Ok(format)
        } else {
            Err(WgpuConversionError::InvalidCombination)
        }
    }
}
