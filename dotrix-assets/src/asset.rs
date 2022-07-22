use dotrix_types as types;

/// Asset control abstraction trait
pub trait Asset: types::id::NameSpace + Send + 'static {
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

    /// Returns name of the asset
    fn namespace(&self) -> u64;
}

impl dyn Asset {
    /// Returns true if the asset is of type T
    #[inline]
    pub fn is<T: Asset>(&self) -> bool {
        let t = std::any::TypeId::of::<T>();
        let concrete = self.type_id();
        t == concrete
    }

    /// Checks asset type and downcasts dynamic reference
    #[inline]
    pub fn downcast_ref<T: Asset>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn Asset as *const T)) }
        } else {
            None
        }
    }

    /// Checks asset type and downcasts dynamic mutable reference
    #[inline]
    pub fn downcast_mut<T: Asset>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn Asset as *mut T)) }
        } else {
            None
        }
    }
}

/// Data structure indicating import request for an asset file
pub struct File {
    /// Path to asset file
    pub path: std::path::PathBuf,
    /// File size, used for buffer initial size
    pub size: usize,
}

/// Resulting data of an asset file import
pub struct Bundle {
    /// Path to asset file
    pub path: std::path::PathBuf,
    /// List of tuple of namespace number and imported assets
    pub assets: Vec<Box<dyn Asset>>,
}

/// Imported asset file report
#[derive(Clone)]
pub struct Resource {
    /// Path to asset file
    pub path: std::path::PathBuf,
    /// Incremental file version (from startup)
    pub version: u32,
    /// List type and name pairs of imported assets
    pub assets: Vec<(String, String)>,
}

impl Resource {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            version: 0,
            assets: Vec::new(),
        }
    }
}
