mod asset;
mod loader;
mod tasks;

use dotrix_core as dotrix;
use dotrix_types::{Id, IdMap};

pub use asset::{Asset, File, Info, Resource};
pub use loader::{LoadError, Loader};
pub use tasks::{LoadTask, StoreTask, Watchdog};

pub const ASSETS_NAMESPACE: u64 = dotrix::NAMESPACE | 0x0100;

/// Assets management service
pub struct Assets {
    root: std::path::PathBuf,
    /// Id indexed assets map
    map: IdMap<Box<dyn Asset>>,
    /// Resources registry indexed by file path
    registry: std::collections::HashMap<std::path::PathBuf, ResourceEntry>,
    /// Index of IDs assigned by asset name
    index: std::collections::HashMap<String, uuid::Uuid>,
    /// Asset Loaders
    loaders: std::collections::HashMap<std::any::TypeId, Box<dyn Loader>>,
    /// Id counter
    last_id: u64,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum ResourceState {
    Idle,
    Loaded,
    Failed,
}

struct ResourceEntry {
    last_modified: Option<std::time::Instant>,
    version: u32,
    state: ResourceState,
}

impl Assets {
    /// Constructs new [`Assets`] instance
    pub fn new() -> Self {
        let root = std::env::current_dir().expect("Current working directory must be accessible");
        Self::new_with_root(root)
    }

    /// Constructs new [`Assets`] instance with custom root
    pub fn new_with_root(root: std::path::PathBuf) -> Self {
        Self {
            root,
            map: IdMap::new(),
            registry: std::collections::HashMap::new(),
            index: std::collections::HashMap::new(),
            loaders: std::collections::HashMap::new(),
            last_id: 0,
        }
    }

    /// Installs new asset loader
    pub fn install<T: Loader>(&mut self, loader: T) {
        self.loaders.insert(std::any::TypeId::of::<T>(), Box::new(loader));
    }

    /// Uninstalls asset loader
    pub fn uninstall<T: Loader>(&mut self) -> Option<T> {
        self.loaders
            .remove(&std::any::TypeId::of::<T>())
            .map(|mut l| {
                *(unsafe {
                    Box::from_raw((Box::leak(l) as * mut dyn Loader) as *mut T)
                })
            })
    }

    /// Returns assets root folder path
    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    /// Sets assets root folder path
    pub fn set_path(&mut self, root: std::path::PathBuf) {
        self.root = root;
    }

    /// Imports an asset file by its relative path and returns [`Id`] of the [`Resource`]
    pub fn import(&mut self, path_str: &str) {
        let path = self.root.as_path().join(path_str);
        self.import_from(path)
    }

    /// Imports an asset file from specified absolute or relative path and returns [`Id`] of the
    /// [`Resource`]
    pub fn import_from(&mut self, path: std::path::PathBuf) {
        /* let name = path
            .file_stem()
            .map(|n| n.to_str().unwrap())
            .unwrap()
            .to_string();*/
        self.registry.insert(path, ResourceEntry {
            last_modified: None,
            version: 0,
            state: ResourceState::Idle,
        });
    }

    /// Associates an asset name with [`Id`] and returns it
    pub fn register<T: Asset>(&mut self, name: &str) -> Id<T> {
        use dotrix_types::id::NameSpace;

        if let Some(uuid) = self.index.get(name) {
            return Id::from(*uuid);
        }

        let id = T::id(self.next_id());
        self.index.insert(name.to_string(), id.uuid());
        id
    }

    fn next_id(&mut self) -> u64 {
        self.last_id += 1;
        self.last_id
    }
}

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Extension {
    pub hot_reload: bool,
}

impl Default for Extension {
    fn default() -> Self {
        Self { hot_reload: false }
    }
}

impl dotrix::Extension for Extension {
    fn setup(self, manager: &mut dotrix::Manager) {
        manager.store(Assets::new());
        manager.schedule(LoadTask::default());
        manager.schedule(StoreTask::default());
        manager.schedule(Watchdog {
            hot_reload: self.hot_reload,
            ..Default::default()
        });
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    struct DummyLoader(u32);

    impl Loader for DummyLoader {
        fn can_load(&self, _: &str) -> bool {
            false
        }

        fn load(&self, path: &std::path::Path) -> Result<Resource, LoadError> {
            Err(LoadError::OpenFile)
        }
    }

    #[test]
    fn install_uninstall_loader() {
        let control_value = 57651236;
        let mut assets = Assets::new();
        assets.install(DummyLoader(control_value));
        let loader: Option<DummyLoader> = assets.uninstall();
        assert_eq!(loader.is_some(), true);
        assert_eq!(loader.unwrap().0, control_value);
    }
}
