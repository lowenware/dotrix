use dotrix_core::assets::{Assets, Mesh};
use dotrix_core::reloadable::*;
use dotrix_core::renderer::Buffer;
use dotrix_core::{Id, Renderer, Transform};

use dotrix_derive::*;

/// Model component
#[derive(Reloadable, BufferProvider)]
#[buffer_provider(field = "transform")]
pub struct Model {
    /// [`Id`] of a [`Mesh`] asset
    pub mesh: Id<Mesh>,
    /// Model transformation uniform
    pub transform: Buffer,
    /// The reload state
    pub reload_state: ReloadState,
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
        self.flag_update();
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            mesh: Id::default(),
            transform: Buffer::uniform("Model Transform Matrix"),
            reload_state: Default::default(),
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
