//! Holds various structure mappings between dotrix and wgpu
//!
//! These are done so that we can limit how much source must be
//! rewritten if wgpu changes any names.
//!

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
