//! Transformation structure and builder
use dotrix_math::{Mat4, Quat, Rad, Rotation3, Vec3};

/// Agregator for skin transformations
#[derive(Default)]
pub struct Builder {
    /// Optional translation vector
    pub translate: Option<Vec3>,
    /// Optional rotation quaternion
    pub rotate: Option<Quat>,
    /// Optional scale vector
    pub scale: Option<Vec3>,
}

impl Builder {
    /// Constructs the builder from translation vector
    #[must_use]
    pub fn with_translate(mut self, translate: Vec3) -> Self {
        self.translate = Some(translate);
        self
    }

    /// Constructs the builder from rotation quaternion
    #[must_use]
    pub fn with_rotate(mut self, rotate: Quat) -> Self {
        self.rotate = Some(rotate);
        self
    }

    /// Constructs the builder from scale vector
    #[must_use]
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = Some(scale);
        self
    }

    /// Builds the transformation
    pub fn build(self) -> Transform {
        let mut transform = Transform::default();
        if let Some(translate) = self.translate {
            transform.translate = translate;
        }
        if let Some(rotate) = self.rotate {
            transform.rotate = rotate;
        }
        if let Some(scale) = self.scale {
            transform.scale = scale;
        }
        transform
    }
}

/// Model transformation structure
#[derive(Debug, Copy, Clone)]
pub struct Transform {
    /// Translation vector
    pub translate: Vec3,
    /// Rotation quaternion
    pub rotate: Quat,
    /// Scale vector
    pub scale: Vec3,
}

impl Transform {
    /// Constructs new transformation with values that won't change the model
    pub fn new() -> Self {
        Self {
            translate: Vec3::new(0.0, 0.0, 0.0),
            rotate: Quat::from_angle_y(Rad(0.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Constructs transformation builder
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Returns transformation matrix
    pub fn matrix(&self) -> Mat4 {
        let t = Mat4::from_translation(self.translate);
        let r = Mat4::from(self.rotate);
        let s = Mat4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        t * r * s
    }

    /// Merge current transformation with values from the [`Builder`] respectively
    #[must_use]
    pub fn merge(&self, builder: &Builder) -> Self {
        Self {
            translate: builder.translate.unwrap_or(self.translate),
            rotate: builder.rotate.unwrap_or(self.rotate),
            scale: builder.scale.unwrap_or(self.scale),
        }
    }

    /// Constructs new transformation from the translation vector
    pub fn from_translation(translate: Vec3) -> Self {
        Self {
            translate,
            ..Default::default()
        }
    }

    /// Constructs new transformation from the rotation quaternion
    pub fn from_rotation(rotate: Quat) -> Self {
        Self {
            rotate,
            ..Default::default()
        }
    }

    /// Constructs new transformation from the scale vector
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Constructs new transformation from the scale factor
    pub fn from_scale_factor(scale: f32) -> Self {
        Self::from_scale(Vec3::new(scale, scale, scale))
    }
}

/*
 * impl From<gltf::scene::Transform> for Transform {
 *    fn from(transform: gltf::scene::Transform) -> Self {
 *        let (translation, rotation, scale) = transform.decomposed();
 *        Self {
 *            translate: Vec3::from(translation),
 *            rotate: Quat::new(rotation[3], rotation[0], rotation[1], rotation[2]),
 *            scale: Vec3::from(scale),
 *        }
 *    }
 * }
 */

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
