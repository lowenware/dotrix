use dotrix_core::assets::{Assets, Mesh};
use dotrix_core::renderer::Buffer;
use dotrix_core::{Id, Renderer, Transform};

/// Model component
pub struct Model {
    /// [`Id`] of a [`Mesh`] asset
    pub mesh: Id<Mesh>,
    /// Model transformation uniform
    pub transform: Buffer,
}

impl Model {
    /// Loads the [`Model`] into GPU buffers
    pub fn load(&mut self, renderer: &Renderer, assets: &mut Assets) -> bool {
        if let Some(mesh) = assets.get_mut(self.mesh) {
            mesh.load(renderer);
        } else {
            return false;
        }
        true
    }
    /// Loads data to the transformation buffer
    pub fn transform(&mut self, renderer: &Renderer, transform: &Transform) {
        let transform_matrix = transform.matrix();
        let transform_raw = AsRef::<[f32; 16]>::as_ref(&transform_matrix);
        renderer.load_buffer(&mut self.transform, bytemuck::cast_slice(transform_raw));
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            mesh: Id::default(),
            transform: Buffer::uniform("Model Transform Matrix"),
        }
    }
}

impl From<Id<Mesh>> for Model {
    fn from(mesh: Id<Mesh>) -> Self {
        Self {
            mesh,
            ..Default::default()
        }
    }
}
