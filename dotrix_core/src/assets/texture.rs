//! Texture asset
use crate::renderer::{Renderer, TextureBuffer, TextureUsages};

/// Texture asset
pub struct Texture {
    /// Texture width in pixels
    pub width: u32,
    /// Texture height in pixels
    pub height: u32,
    /// Texture depth
    pub depth: u32,
    /// Raw texture data
    pub data: Vec<u8>,
    /// Permitted texture usages
    pub usages: TextureUsages,
    /// Texture buffer
    pub buffer: TextureBuffer,
    /// Was the asset changed
    pub changed: bool,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            depth: 0,
            data: vec![],
            usages: TextureUsages::create().texture().write(),
            buffer: Default::default(),
            changed: false,
        }
    }
}

impl Texture {
    /// Loads the [`Texture`] data to a buffer
    pub fn load(&mut self, renderer: &Renderer) {
        if !self.changed && self.buffer.loaded() {
            return;
        }

        renderer.load_texture_buffer_with_usage(
            &mut self.buffer,
            self.width,
            self.height,
            &[&self.data],
            self.usages,
        );
        self.changed = false;
    }

    /// Unloads the [`Texture`] data from the buffer
    pub fn unload(&mut self) {
        self.buffer.unload();
    }
}
