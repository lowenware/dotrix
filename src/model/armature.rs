use dotrix_assets as assets;
use dotrix_math::Mat4;
use dotrix_types::id;
use dotrix_types::{Id, Transform};
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct Armature {
    pub label: String,
    pub joints: HashMap<Id<Joint>, Joint>,
    pub names: HashMap<String, Id<Joint>>,
    pub index: Vec<Id<Joint>>,
}

impl Armature {
    pub fn new(label: String, capacity: usize) -> Self {
        Self {
            label,
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

impl id::NameSpace for Armature {
    fn namespace() -> u64 {
        assets::NAMESPACE | 0x11
    }
}

impl assets::Asset for Armature {
    fn name(&self) -> &str {
        &self.label
    }

    fn namespace(&self) -> u64 {
        <Self as id::NameSpace>::namespace()
    }
}
