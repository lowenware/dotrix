//! Texture asset
use crate::renderer::{Renderer, Texture as TextureBuffer};

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
            buffer: TextureBuffer::new("Texture"),
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

        renderer.load_texture(&mut self.buffer, self.width, self.height, &[&self.data]);
        self.changed = false;
    }

    /// Unloads the [`Texture`] data from the buffer
    pub fn unload(&mut self) {
        self.buffer.unload();
    }

    /// Fetch data from the gpu
    ///
    /// This is useful textures that are altered on the gpu
    ///
    /// This operation is slow and should mostly be
    /// used for debugging
    pub fn fetch_from_gpu(
        &mut self,
        renderer: &mut Renderer,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, wgpu::BufferAsyncError>> {
        renderer.fetch_texture(&self.buffer, [self.width, self.height, self.depth])
    }
}
