use dotrix_core::Id;
use dotrix_core::assets::{ Mesh, Texture };

/// Widget
#[derive(Default)]
pub struct Widget {
    /// Widget mesh
    pub mesh: Mesh,
    /// Id of the widget texture
    pub texture: Id<Texture>,
}
