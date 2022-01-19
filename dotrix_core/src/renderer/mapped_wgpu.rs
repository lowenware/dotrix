//! Holds various structure mappings between dotrix and wgpu
//!
//! These are done so that we can limit how much source must be
//! rewritten if wgpu changes any names.
//!

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use thiserror::Error;
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

/// Error that can occur when no applicable wgpu texture format can be
/// found
#[derive(Error, Debug)]
pub enum WgpuConversionError {
    #[error("Number of channel bits is not supported")]
    InvalidChannelBits,
    #[error("Channel format is not supported with this combination of bits/channels")]
    InvalidChannelFormat,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum DataSize {
    Bits8,
    Bits16,
    Bits32,
}

/// The possible color channel formats,
/// not every combination of channel_bits
/// and channel_format is valid
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum ChannelFormat {
    /// Unsigned interger normalised to float 0..1. in the shader
    Unorm,
    /// Signed interger normalised to float 0..1. in the shader
    Snorm,
    /// Unsigned integer
    Uint,
    /// Signed integer
    Sint,
    /// Full floating point
    Float,
    /// Unsigned interger normalised to float 0..1. in the shader in srgb color space
    UnormSrgb,
}

/// Defines the texture format
///
/// Not all combinations can be represented in wgpu.
/// You can use `TextureFormat::into_wgpu` to ensure that
/// it is valid prior to passing it into your textures
#[derive(Copy, Clone, Debug)]
pub enum TextureFormat {
    /// Single channel
    R {
        /// There are only 3 supported channel_bits 8, 16 and 32
        channel_bits: usize,
        /// The format of all the channels
        channel_format: ChannelFormat,
    },
    /// Two channel
    Rg {
        /// There are only 3 supported channel_bits 8, 16 and 32
        channel_bits: usize,
        /// The format of all the channels
        channel_format: ChannelFormat,
    },
    /// Four channel
    Rgba {
        /// There are only 3 supported channel_bits 8, 16 and 32
        channel_bits: usize,
        /// The format of all the channels
        channel_format: ChannelFormat,
    },
    /// Raw wgpu format that is asserted to be
    /// unfilterable
    Wgpu(WgpuTextureFormat),
    /// Raw wgpu format that is asserted to be
    /// filterable it is up to the end user
    /// to ensure that the type is filterable
    WgpuFilterable(WgpuTextureFormat),
}

impl TextureFormat {
    /// Will attempt to convert the format into a concrete
    /// `[wgpu::TextureFormat]` this can be used to ensure that
    /// your chosen format is avaliable with an error you can
    /// respond to. If successful the result can be
    /// `[TextureFormat::Wgpu](TheWgpuTextureFormat)`
    pub fn into_wgpu(self) -> Result<Self, WgpuConversionError> {
        Ok(Self::Wgpu(self.try_into()?))
    }

    /// Only certain formats can be used in a filtering
    /// sampler. This will check if it is one of these kinds
    pub(crate) fn filterable(&self) -> bool {
        let channel_format = match self {
            TextureFormat::R { channel_format, .. } => channel_format,
            TextureFormat::Rg { channel_format, .. } => channel_format,
            TextureFormat::Rgba { channel_format, .. } => channel_format,
            TextureFormat::Wgpu(_) => return false,
            TextureFormat::WgpuFilterable(_) => return true,
        };
        match channel_format {
            ChannelFormat::Unorm => true,
            ChannelFormat::Snorm => true,
            ChannelFormat::Uint => false,
            ChannelFormat::Sint => false,
            // Float can be filterable if the `wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES`
            // is enabled
            ChannelFormat::Float => false,
            ChannelFormat::UnormSrgb => true,
        }
    }
}

lazy_static! {
    // Hashmap order is, channels => size => format
    static ref TEX_FORMAT_MAP: HashMap<u8, HashMap<DataSize, HashMap<ChannelFormat, WgpuTextureFormat>>> = {
        HashMap::from([
            // 1 Channel (Red)
            (1,
                HashMap::from([
                    (DataSize::Bits8,
                        HashMap::from([
                            (ChannelFormat::Unorm,WgpuTextureFormat::R8Unorm),
                            (ChannelFormat::Snorm,WgpuTextureFormat::R8Snorm),
                            (ChannelFormat::Uint,WgpuTextureFormat::R8Uint),
                            (ChannelFormat::Sint,WgpuTextureFormat::R8Sint),
                        ])
                    ),
                    (DataSize::Bits16,
                        HashMap::from([
                            (ChannelFormat::Unorm,WgpuTextureFormat::R16Unorm),
                            (ChannelFormat::Snorm,WgpuTextureFormat::R16Snorm),
                            (ChannelFormat::Uint,WgpuTextureFormat::R16Uint),
                            (ChannelFormat::Sint,WgpuTextureFormat::R16Sint),
                            (ChannelFormat::Float,WgpuTextureFormat::R16Float),
                        ])
                    ),
                    (DataSize::Bits32,
                        HashMap::from([
                            (ChannelFormat::Uint,WgpuTextureFormat::R32Uint),
                            (ChannelFormat::Sint,WgpuTextureFormat::R32Sint),
                            (ChannelFormat::Float,WgpuTextureFormat::R32Float),
                        ])
                    )
                ]),
            ),
            // 2 Channel (Red, Green)
            (2,
                HashMap::from([
                    (DataSize::Bits8,
                        HashMap::from([
                            (ChannelFormat::Unorm,WgpuTextureFormat::Rg16Unorm),
                            (ChannelFormat::Snorm,WgpuTextureFormat::Rg16Snorm),
                            (ChannelFormat::Uint,WgpuTextureFormat::Rg16Uint),
                            (ChannelFormat::Sint,WgpuTextureFormat::Rg16Sint),
                            (ChannelFormat::Float,WgpuTextureFormat::Rg16Float),
                        ])
                    ),
                    (DataSize::Bits16,
                        HashMap::from([
                            (ChannelFormat::Unorm,WgpuTextureFormat::Rg8Unorm),
                            (ChannelFormat::Snorm,WgpuTextureFormat::Rg8Snorm),
                            (ChannelFormat::Uint,WgpuTextureFormat::Rg8Uint),
                            (ChannelFormat::Sint,WgpuTextureFormat::Rg8Sint),
                        ])
                    ),
                    (DataSize::Bits32,
                        HashMap::from([
                            (ChannelFormat::Uint,WgpuTextureFormat::Rg32Uint),
                            (ChannelFormat::Sint,WgpuTextureFormat::Rg32Sint),
                            (ChannelFormat::Float,WgpuTextureFormat::Rg32Float),
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
                            (ChannelFormat::Unorm,WgpuTextureFormat::Rgba8Unorm),
                            (ChannelFormat::Snorm,WgpuTextureFormat::Rgba8Snorm),
                            (ChannelFormat::Uint,WgpuTextureFormat::Rgba8Uint),
                            (ChannelFormat::Sint,WgpuTextureFormat::Rgba8Sint),
                            (ChannelFormat::UnormSrgb,WgpuTextureFormat::Rgba8UnormSrgb),
                        ]),
                    ),
                    (DataSize::Bits16,
                        HashMap::from([
                            (ChannelFormat::Unorm,WgpuTextureFormat::Rgba16Unorm),
                            (ChannelFormat::Snorm,WgpuTextureFormat::Rgba16Snorm),
                            (ChannelFormat::Uint,WgpuTextureFormat::Rgba16Uint),
                            (ChannelFormat::Sint,WgpuTextureFormat::Rgba16Sint),
                            (ChannelFormat::Float,WgpuTextureFormat::Rgba16Float),
                        ])
                    ),
                    (DataSize::Bits32,
                        HashMap::from([
                            (ChannelFormat::Uint,WgpuTextureFormat::Rgba32Uint),
                            (ChannelFormat::Sint,WgpuTextureFormat::Rgba32Sint),
                            (ChannelFormat::Float,WgpuTextureFormat::Rgba32Float),
                        ])
                    ),
                ]),
            ),
        ])
    };
}

impl TryFrom<TextureFormat> for WgpuTextureFormat {
    type Error = WgpuConversionError;
    fn try_from(value: TextureFormat) -> Result<Self, Self::Error> {
        let (channels, channel_bits, format) = match &value {
            TextureFormat::R {
                channel_bits,
                channel_format,
            } => (1, channel_bits, channel_format),
            TextureFormat::Rg {
                channel_bits,
                channel_format,
            } => (2, channel_bits, channel_format),
            TextureFormat::Rgba {
                channel_bits,
                channel_format,
            } => (4, channel_bits, channel_format),
            TextureFormat::Wgpu(wgpu) => return Ok(*wgpu),
            TextureFormat::WgpuFilterable(wgpu) => return Ok(*wgpu),
        };
        let data_size = match channel_bits {
            8 => DataSize::Bits8,
            16 => DataSize::Bits16,
            32 => DataSize::Bits32,
            _ => return Err(WgpuConversionError::InvalidChannelBits),
        };
        if let Some(Some(Some(&wgpu))) = TEX_FORMAT_MAP.get(&(channels as u8)).map(|size_map| {
            size_map
                .get(&data_size)
                .map(|format_map| format_map.get(format))
        }) {
            Ok(wgpu)
        } else {
            Err(WgpuConversionError::InvalidChannelFormat)
        }
    }
}
