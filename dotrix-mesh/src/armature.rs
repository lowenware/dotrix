use dotrix_assets as assets;
use dotrix_types::id;
use dotrix_types::{Id, Transform};
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct Node {
    pub label: Option<String>,
    pub parent: Id<Node>,
    pub local_bind_transform: Transform,
}

#[derive(Default, Debug, Clone)]
pub struct Armature {
    pub label: String,
    pub bones: HashMap<Id<Node>, Node>,
    pub index: HashMap<String, Id<Node>>,
}

impl Armature {
    pub fn new(label: String) -> Self {
        Self {
            label,
            bones: HashMap::new(),
            index: HashMap::new(),
        }
    }
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
