
pub struct Animation {
    pub keyframes: Vec<f32>,
    pub joint_transforms: Vec<JointTransforms>,
}

pub struct JointTransforms {
    pub scales: Option<Vec<cgmath::Vector3<f32>>>,
    pub rotations: Option<Vec<cgmath::Quaternion<f32>>>,
    pub translations: Option<Vec<cgmath::Vector3<f32>>>,
}

impl Animation {
    pub fn new(
        keyframes: Vec<f32>,
        joint_transforms: Vec<JointTransforms>,
    ) -> Self {
        Self {
            keyframes,
            joint_transforms,
        }
    }
}

impl JointTransforms {
    pub fn new() -> Self {
        Self {
            scales: None,
            rotations: None,
            translations: None,
        }
    }
}

impl Default for JointTransforms {
    fn default() -> Self {
        Self::new()
    }
}
