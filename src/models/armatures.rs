use std::collections::HashMap;

use super::{Transform3D, TransformBuilder};
use crate::loaders::Asset;
use crate::math::Mat4;
use crate::utils::Id;

#[derive(Default, Debug, Clone)]
pub struct Armature {
    pub name: String,
    pub joints: HashMap<Id<Joint>, Joint>,
    pub names: HashMap<String, Id<Joint>>,
    pub index: Vec<Id<Joint>>,
}

impl Armature {
    pub fn new(name: String, capacity: usize) -> Self {
        Self {
            name,
            joints: HashMap::with_capacity(capacity),
            names: HashMap::with_capacity(capacity),
            index: Vec::with_capacity(capacity),
        }
    }

    pub fn add(&mut self, id: Id<Joint>, name: Option<String>, joint: Joint) -> Id<Joint> {
        self.index.push(id);
        if let Some(name) = name {
            self.names.insert(name, id);
        }
        self.joints.insert(id, joint);
        id
    }

    pub fn transform(
        &self,
        // model_transform: &Mat4,
        joint_local_transforms: Option<HashMap<Id<Joint>, TransformBuilder>>,
    ) -> Vec<Mat4> {
        let mut result = vec![Mat4::IDENTITY; self.joints.len()];
        for (joint_index, joint_id) in self.index.iter().enumerate() {
            let joint = self.joints.get(joint_id).expect("Can't find indexed joint");
            let parent_transform = joint
                .parent_id
                .as_ref()
                .and_then(|&parent_id| self.index.iter().position(|i| *i == parent_id))
                .map(|parent_index| result[parent_index])
                .unwrap_or_else(|| Mat4::IDENTITY);

            let local_joint_transform = joint_local_transforms
                .as_ref()
                .and_then(|transforms| transforms.get(joint_id));

            let global_joint_transform = parent_transform
                * local_joint_transform
                    .map(|l| joint.local_bind_transform.merge(l))
                    .as_ref()
                    .unwrap_or(&joint.local_bind_transform)
                    .matrix();

            result[joint_index] = global_joint_transform;
            // joint
            //    .inverse_bind_matrix
            //    .as_ref()
            //    .map(|&inverse_bind_matrix| global_joint_transform * inverse_bind_matrix)
            //    .unwrap_or(global_joint_transform);
        }

        result
            .into_iter()
            .enumerate()
            .map(|(index, global_joint_transform)| {
                let joint_id = self.index[index];
                let joint = self
                    .joints
                    .get(&joint_id)
                    .expect("Can't find joint by index");
                joint
                    .inverse_bind_matrix
                    .as_ref()
                    .map(|&inverse_bind_matrix| global_joint_transform * inverse_bind_matrix)
                    .unwrap_or_else(|| global_joint_transform)
            })
            .collect::<Vec<_>>()
    }
}
/*
pub fn transform(
        &self,
        skin_transform: &mut Pose,
        model_transform: &Mat4,
        local_transforms: Option<HashMap<JointId, TransformBuilder>>,
    ) {
        for (i, joint) in self.joints.iter().enumerate() {
            let parent_transform = joint
                .parent_id
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

    */

#[derive(Default, Debug, Clone)]
/// Joint data structure
pub struct Joint {
    /// Local bind transformation of the joint
    pub local_bind_transform: Transform3D,
    /// Inverse bind matrix of the [`Joint`]
    pub inverse_bind_matrix: Option<Mat4>,
    /// Parent joint Id
    pub parent_id: Option<Id<Joint>>,
}

impl Asset for Armature {
    fn name(&self) -> &str {
        &self.name
    }
}
