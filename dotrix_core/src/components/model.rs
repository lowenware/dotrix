use crate::{
    assets::{ Assets, Mesh },
    generics::Id,
    renderer::Renderer,
};

/// Model component
#[derive(Default)]
pub struct Model {
    /// [`Id`] of a [`Mesh`] asset
    pub mesh: Id<Mesh>,
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
}
