//! Generic asset resource

/// Data structure representing an asset file
pub struct Resource {
    name: String,
    path: String,
}

impl Resource {
    /// Constructs new resource
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
        }
    }

    /// Returns the [`Resource`] path
    pub fn path(&self) -> &String {
        &self.path
    }

    /// Returns the [`Resource`] name
    pub fn name(&self) -> &String {
        &self.name
    }
}

