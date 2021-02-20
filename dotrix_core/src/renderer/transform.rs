use dotrix_math::{Mat4, Vec3, Quat, Rotation3, Rad};

#[derive(Default)]
pub struct TransformBuilder {
    pub translate: Option<Vec3>,
    pub rotate: Option<Quat>,
    pub scale: Option<Vec3>,
}

impl TransformBuilder {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translate: Some(translation),
            rotate: None,
            scale: None
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            translate: None,
            rotate: Some(rotation),
            scale: None
        }
    }

    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            translate: None,
            rotate: None,
            scale: Some(scale) 
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub translate: Vec3,
    pub rotate: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            translate: Vec3::new(0.0, 0.0, 0.0),
            rotate: Quat::from_angle_y(Rad(0.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn matrix(&self) -> Mat4 {
        let t = Mat4::from_translation(self.translate);
        let r = Mat4::from(self.rotate);
        let s = Mat4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        t * r * s
    }

    pub fn merge(&self, builder: &TransformBuilder) -> Self {
        Self {
            translate: builder.translate.unwrap_or(self.translate),
            rotate: builder.rotate.unwrap_or(self.rotate),
            scale: builder.scale.unwrap_or(self.scale),
        }
    }

    pub fn from_translation(translate: Vec3) -> Self {
        Self {
            translate,
            ..Default::default()
        }
    }

    pub fn from_rotation(rotate: Quat) -> Self {
        Self {
            rotate,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    pub fn from_scale_factor(scale: f32) -> Self {
        Self::from_scale(Vec3::new(scale, scale, scale))
    }
}

impl From<gltf::scene::Transform> for Transform {
    fn from(transform: gltf::scene::Transform) -> Self {
        let (translation, rotation, scale) = transform.decomposed();
        Self {
            translate: Vec3::from(translation),
            rotate: Quat::new(rotation[3], rotation[0], rotation[1], rotation[2]),
            scale: Vec3::from(scale),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
