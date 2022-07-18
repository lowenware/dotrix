use dotrix_types as types;


/// Asset control abstraction trait
pub trait Asset: types::id::NameSpace + std::any::Any + Send + 'static {
    /// Returns [`std::any::TypeId`] of the asset type
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    /// Returns name of the asset type
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Returns name of the asset
    fn name(&self) -> &str;
}

/// Data structure indicating import request for an asset file
pub struct File {
    /// Path to asset file
    pub path: std::path::PathBuf,
}

/// Resulting data of an asset file import
pub struct Resource {
    /// Path to asset file
    pub path: std::path::PathBuf,
    /// Last modified timestamp
    pub last_modified: Option<std::time::Instant>,
    /// Incremental file version (from startup)
    pub version: u32,
    /// List of imported assets
    pub assets: Vec<Box<dyn Asset>>,
}

/// Imported asset file report
pub struct Info {
    /// Path to asset file
    pub path: std::path::PathBuf,
    /// Last modified timestamp
    pub last_modified: Option<std::time::Instant>,
    /// Incremental file version (from startup)
    pub version: u32,
    /// List type and name pairs of imported assets
    pub assets: Vec<(String, String)>,
}

