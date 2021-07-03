use crate::renderer::{ Renderer, TextureBuffer };

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
    pub buffer: TextureBuffer,
    pub changed: bool,
}

impl Texture {
    /// Loads the [`Texture`] data to a buffer
    pub fn load(&mut self, renderer: &Renderer) {
        if !self.changed && !self.buffer.is_empty() {
            return;
        }

        renderer.load_texture_buffer(&mut self.buffer, self.width, self.height, &self.data);
        self.changed = false;
    }

    /// Unloads the [`Texture`] data from the buffer
    pub fn unload(&mut self) {
        self.buffer.empty();
    }
}
