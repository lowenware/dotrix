use cgmath::{Vector3, Quaternion, Matrix4};

pub struct Transform {
    pub translate: Option<Vector3<f32>>,
    pub rotate: Option<Quaternion<f32>>,
    pub scale: Option<Vector3<f32>>,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            translate: None,
            rotate: None,
            scale: None,
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        use cgmath::SquareMatrix;
        let t = self.translate
            .map(Matrix4::from_translation)
            .unwrap_or_else(Matrix4::identity);
        let r = self.rotate
            .map(Matrix4::from)
            .unwrap_or_else(Matrix4::identity);
        let s = self.scale
            .map(|s| Matrix4::from_nonuniform_scale(s.x, s.y, s.z))
            .unwrap_or_else(Matrix4::identity);
        t * r * s
    }

    pub fn merge(&self, target: &Transform) -> Self {
        Self {
            translate: target.translate.or(self.translate),
            rotate: target.rotate.or(self.rotate),
            scale: target.scale.or(self.scale),
        }
    }

    pub fn from_translation(translation: Vector3<f32>) -> Self {
        Self {
            translate: Some(translation),
            rotate: None,
            scale: None
        }
    }

    pub fn from_rotation(rotation: Quaternion<f32>) -> Self {
        Self {
            translate: None,
            rotate: Some(rotation),
            scale: None
        }
    }

    pub fn from_scale(scale: Vector3<f32>) -> Self {
        Self {
            translate: None,
            rotate: None,
            scale: Some(scale) 
        }
    }
}

impl From<gltf::scene::Transform> for Transform {
    fn from(transform: gltf::scene::Transform) -> Self {
        let (translation, rotation, scale) = transform.decomposed();
        Self {
            translate: Some(Vector3::from(translation)),
            rotate: Some(Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2])),
            scale: Some(Vector3::from(scale)),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
