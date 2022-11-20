use crate::Material;
use dotrix_ecs as ecs;
use dotrix_math::{Quat, Rad, Vec3};
use dotrix_mesh::{Armature, Mesh};
use dotrix_shader::Shader;
use dotrix_types::{Id, Transform};

pub struct Entity {
    pub mesh: Id<Mesh>,
    pub material: Id<Material>,
    pub armature: Id<Armature>,
    pub shader: Id<Shader>,
    pub translate: Vec3,
    pub scale: Vec3,
    pub rotate: Quat,
}

impl From<Entity> for ecs::Entity {
    fn from(entity: Entity) -> Self {
        ecs::Entity::new((
            entity.mesh,
            entity.material,
            entity.armature,
            entity.shader,
            Transform::new(entity.translate, entity.rotate, entity.scale),
        ))
    }
}

impl Default for Entity {
    fn default() -> Self {
        use dotrix_math::Rotation3;

        Self {
            mesh: Id::null(),
            material: Id::null(),
            armature: Id::null(),
            shader: Id::null(),
            translate: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotate: Quat::from_angle_y(Rad(0.0)),
        }
    }
}
