mod animations;
pub use animations::{Animation, Interpolation};

mod armatures;
pub use armatures::{Armature, Joint};

mod colors;
pub use colors::Color;

mod images;
pub use images::{Image, ImageFormat};

mod materials;
pub use materials::Material;

mod meshes;
pub use meshes::{
    AttributeValues, Mesh, VertexAttributeIter, VertexAttributeIterItem, VertexBufferLayout,
};

mod transforms;
pub use transforms::{Transform, TransformBuilder};

mod vertices;
pub use vertices::{
    VertexAttribute, VertexBitangent, VertexJoints, VertexNormal, VertexPosition, VertexTangent,
    VertexTexture, VertexWeights,
};

use crate::math::{Quat, Vec3};
use crate::utils::Id;
use crate::world::Entity;

pub struct Model {
    pub mesh: Id<Mesh>,
    pub material: Id<Material>,
    pub armature: Id<Armature>,
    pub translate: Vec3,
    pub scale: Vec3,
    pub rotate: Quat,
}

impl From<Model> for Entity {
    fn from(model: Model) -> Self {
        Entity::new((
            model.mesh,
            model.material,
            model.armature,
            Transform::new(model.translate, model.rotate, model.scale),
        ))
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            mesh: Id::null(),
            material: Id::null(),
            armature: Id::null(),
            translate: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotate: Quat::IDENTITY,
        }
    }
}
