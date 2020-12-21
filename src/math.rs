use cgmath::{Vector3, Quaternion, Matrix4};

#[derive(Default)]
pub struct TransformBuilder {
    pub translate: Option<Vector3<f32>>,
    pub rotate: Option<Quaternion<f32>>,
    pub scale: Option<Vector3<f32>>,
}

impl TransformBuilder {
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

pub struct Transform {
    pub translate: Vector3<f32>,
    pub rotate: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new() -> Self {
        use cgmath::Rotation3;
        Self {
            translate: Vector3::new(0.0, 0.0, 0.0),
            rotate: Quaternion::from_angle_y(cgmath::Rad(0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        let t = Matrix4::from_translation(self.translate);
        let r = Matrix4::from(self.rotate);
        let s = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        t * r * s
    }

    pub fn merge(&self, builder: &TransformBuilder) -> Self {
        Self {
            translate: builder.translate.unwrap_or(self.translate),
            rotate: builder.rotate.unwrap_or(self.rotate),
            scale: builder.scale.unwrap_or(self.scale),
        }
    }

    pub fn from_translation(translate: Vector3<f32>) -> Self {
        Self {
            translate,
            ..Default::default()
        }
    }

    pub fn from_rotation(rotate: Quaternion<f32>) -> Self {
        Self {
            rotate,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: Vector3<f32>) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    pub fn from_scale_factor(scale: f32) -> Self {
        Self::from_scale(Vector3::new(scale, scale, scale))
    }
}

impl From<gltf::scene::Transform> for Transform {
    fn from(transform: gltf::scene::Transform) -> Self {
        let (translation, rotation, scale) = transform.decomposed();
        Self {
            translate: Vector3::from(translation),
            rotate: Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2]),
            scale: Vector3::from(scale),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

// nalgebra and cgmath lerp methods produce some weird artifacts, see link bellow
// https://github.com/rustgd/cgmath/issues/300
pub fn slerp(
    left: cgmath::Quaternion<f32>,
    right: cgmath::Quaternion<f32>,
    amount: f32
) -> cgmath::Quaternion<f32> {
    let num2;
    let num3;
    let num = amount;
    let mut num4 = (((left.v.x * right.v.x) + (left.v.y * right.v.y)) + (left.v.z * right.v.z))
        + (left.s * right.s);
    let mut flag = false;
    if num4 < 0.0 {
        flag = true;
        num4 = -num4;
    }
    if num4 > 0.999_999 {
        num3 = 1.0 - num;
        num2 = if flag { -num } else { num };
    } else {
        let num5 = num4.acos();
        let num6 = 1.0 / num5.sin();
        num3 = ((1.0 - num) * num5).sin() * num6;
        num2 = if flag {
            -(num * num5).sin() * num6
        } else {
            (num * num5).sin() * num6
        };
    }
    cgmath::Quaternion::new(
        (num3 * left.s) + (num2 * right.s),
        (num3 * left.v.x) + (num2 * right.v.x),
        (num3 * left.v.y) + (num2 * right.v.y),
        (num3 * left.v.z) + (num2 * right.v.z),
    )
}

