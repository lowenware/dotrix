use std::collections::HashMap;

use super::Transform;
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
}

#[derive(Default, Debug, Clone)]
/// Joint data structure
pub struct Joint {
    /// Local bind transformation of the joint
    pub local_bind_transform: Transform,
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
