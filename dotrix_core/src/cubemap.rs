use crate::assets::{Assets, Texture};
use crate::renderer::{Renderer, Texture as TextureBuffer};
use crate::Id;

/// Holds number of faces of a cubemap
pub const FACES_COUNT: usize = 6;

/// Material component
pub struct CubeMap {
    /// Id of the right cube side
    pub right: Id<Texture>,
    /// Id of the left cube side
    pub left: Id<Texture>,
    /// Id of the top cube side
    pub top: Id<Texture>,
    /// Id of the bottom cube side
    pub bottom: Id<Texture>,
    /// Id of the back cube side
    pub back: Id<Texture>,
    /// Id of the front cube side
    pub front: Id<Texture>,
    /// Pipeline buffer
    pub buffer: TextureBuffer,
}

impl Default for CubeMap {
    fn default() -> Self {
        Self {
            right: Id::default(),
            left: Id::default(),
            top: Id::default(),
            bottom: Id::default(),
            back: Id::default(),
            front: Id::default(),
            buffer: TextureBuffer::new_cube("CubeMap Texture Buffer"),
        }
    }
}

impl CubeMap {
    /// Loads the [`CubeMap`] into GPU buffers
    pub fn load(&mut self, renderer: &Renderer, assets: &mut Assets) -> bool {
        if self.loaded() {
            return true;
        }

        let faces = [
            self.right,
            self.left,
            self.top,
            self.bottom,
            self.back,
            self.front,
        ];
        let mut textures = Vec::with_capacity(FACES_COUNT);
        let mut width = 0;
        let mut height = 0;
        for &id in faces.iter() {
            if let Some(texture) = assets.get(id) {
                width = texture.width;
                height = texture.height;
                textures.push(texture.data.as_slice());
            } else {
                return false;
            }
        }

        renderer.load_texture(&mut self.buffer, width, height, textures.as_slice());

        true
    }

    /// Returns true if cubemap buffer was loaded to GPU
    pub fn loaded(&self) -> bool {
        self.buffer.loaded()
    }

    /// Unloads the Cubemap
    pub fn unload(&mut self) {
        self.buffer.unload();
    }
}
