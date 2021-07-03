//! Skin asset
use std::collections::HashMap;
use crate::{
    pose::Pose,
    transform::{ Transform, Builder as TransformBuilder },
};
use dotrix_math::{ Mat4, SquareMatrix };

/// Maximal number of joints in skin
pub const MAX_JOINTS: usize = 32;

/// Joint identificator (unsigned integer)
pub type JointId = usize;

/// Joint data structure
pub struct Joint {
    /// Local bind transformation of the joint
    pub local_bind_transform: Transform,
    /// name of the joint
    pub name: Option<String>,
    /// Id of the joint (defined by model)
    pub id: JointId,
    /// Parent joint Id
    pub parent_id: Option<JointId>,
}

impl Joint {
    /// Constructs new [`Joint`]
    pub fn new(
        id: JointId,
        parent_id: Option<JointId>,
        name: Option<String>,
        local_bind_transform: Transform,
    ) -> Self {
        Self { local_bind_transform, name, id, parent_id }
    }

    fn transform(
        &self,
        parent_transform: &Mat4,
        local_transform: Option<&TransformBuilder>,
    ) -> JointTransform {
        let local_transform = local_transform
            .map(|l| self.local_bind_transform.merge(l))
            .as_ref()
            .unwrap_or(&self.local_bind_transform)
            .matrix();

        JointTransform {
            id: self.id,
            global_transform: parent_transform * local_transform
        }
    }
}

/// Transformation of the joint
#[derive(Debug, Clone)]
pub struct JointTransform {
    /// Id of a joint
    pub id: JointId,
    /// Global transformation of the joint
    pub global_transform: Mat4,
}

impl Default for JointTransform {
    fn default() -> Self {
        Self {
            id: 0,
            global_transform: Mat4::identity(),
        }
    }
}

/// Joints inverse bind matrix index
pub struct JointIndex {
    /// Id of the [`Joint`]
    pub id: JointId,
    /// Inverse bind matrix of the [`Joint`]
    pub inverse_bind_matrix: Option<Mat4>,
}

/// Model skin attribute
#[derive(Default)]
pub struct Skin {
    /// List of all skin joints (the order does matter)
    pub joints: Vec<Joint>,
    /// Joints inverse bind matrix index
    pub index: Vec<JointIndex>,
}

impl Skin {
    /// Constructs new [`Skin`]
    pub fn new(
        joints: Vec<Joint>,
        mut index: Vec<JointIndex>,
        inverse_bind_matrices: Option<Vec<Mat4>>,
    ) -> Self {

        if let Some(inverse_bind_matrices) = inverse_bind_matrices {
            for (mut joint_index, matrix) in index.iter_mut().zip(inverse_bind_matrices.iter()) {
                joint_index.inverse_bind_matrix = Some(*matrix);
            }
        }

        Self {
            joints,
            index,
        }
    }

    fn index(&self, joint_id: JointId) -> usize {
        self.joints.iter().position(|j| j.id == joint_id).unwrap()
    }

    /// Applies `local_transforms` to the [`Skin`] and stores output in the [`Pose`]
    pub fn transform(
        &self,
        skin_transform: &mut Pose,
        model_transform: &Mat4,
        local_transforms: Option<HashMap<JointId, TransformBuilder>>,
    ) {

        for (i, joint) in self.joints.iter().enumerate() {
            let parent_transform = joint.parent_id
                .map(|parent_id| skin_transform.joints[self.index(parent_id)].global_transform)
                .or(Some(*model_transform))
                .unwrap();

            let local_transform = local_transforms
                .as_ref()
                .map(|l| l.get(&joint.id))
                .unwrap_or(None);

            if i < skin_transform.joints.len() {
                skin_transform.joints[i] = joint.transform(&parent_transform, local_transform);
            } else {
                panic!("Joints count exceeds limit of {:?}", MAX_JOINTS);
            }
        }
    }
}


