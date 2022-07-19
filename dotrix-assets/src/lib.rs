mod asset;
mod loader;
mod tasks;

use dotrix_core as dotrix;
use dotrix_types::Id;

pub use asset::{Asset, Bundle, File, Resource};
pub use loader::{LoadError, Loader};
pub use tasks::{LoadTask, StoreTask, Watchdog};

pub const ASSETS_NAMESPACE: u64 = dotrix::NAMESPACE | 0x0100;

/// Assets management service
pub struct Assets {
    /// Assets root folder path
    root: std::path::PathBuf,
    /// Index of IDs assigned by asset name
    registry: std::collections::HashMap<String, uuid::Uuid>,
    /// Id indexed assets map
    map: std::collections::HashMap<uuid::Uuid, Box<dyn Asset>>,
    /// Resources registry indexed by file path
    resources: std::collections::HashMap<std::path::PathBuf, Resource>,
    /// Asset Loaders
    loaders: std::collections::HashMap<std::any::TypeId, Box<dyn Loader>>,
    /// Id counter
    last_id: u64,
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
            registry: std::collections::HashMap::new(),
            map: std::collections::HashMap::new(),
            resources: std::collections::HashMap::new(),
            loaders: std::collections::HashMap::new(),
            last_id: 0,
        }
    }

    /// Installs new asset loader
    pub fn install<T: Loader>(&mut self, loader: T) {
        self.loaders
            .insert(std::any::TypeId::of::<T>(), Box::new(loader));
    }

    /// Uninstalls asset loader
    pub fn uninstall<T: Loader>(&mut self) -> Option<T> {
        self.loaders
            .remove(&std::any::TypeId::of::<T>())
            .map(|l| *(unsafe { Box::from_raw((Box::leak(l) as *mut dyn Loader) as *mut T) }))
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
        self.resources.insert(path.clone(), Resource::new(path));
    }

    /// Associates an asset name with [`Id`] and returns it
    pub fn register<T: Asset>(&mut self, name: &str) -> Id<T> {
        if let Some(uuid) = self.registry.get(name) {
            return Id::from(*uuid);
        }

        let id = T::id(self.next_id());
        self.registry.insert(name.to_string(), *id.uuid());
        id
    }

    /// Stores an asset under user defined name and returns [`Id`] of it
    pub fn store_as<T: Asset>(&mut self, asset: T, name: &str) -> Id<T> {
        let id = self.register(name);
        self.map.insert(*id.uuid(), Box::new(asset));
        id
    }

    /// Stores an asset and returns [`Id`] of it
    pub fn store<T: Asset>(&mut self, asset: T) -> Id<T> {
        let id = T::id(self.next_id());
        self.map.insert(*id.uuid(), Box::new(asset));
        id
    }

    /// Searches for an asset by the name and return [`Id`] of it if the asset exists
    pub fn find<T: Asset>(&self, name: &str) -> Option<Id<T>> {
        self.registry.get(name).map(|uuid| Id::from(*uuid))
    }

    /// Searches an asset by its [`Id`] and returns it by a reference if the asset exists
    pub fn get<T: Asset>(&self, id: Id<T>) -> Option<&T> {
        self.map
            .get(id.uuid())
            .map(|a| a.downcast_ref::<T>())
            .unwrap_or(None)
    }

    /// Searches an asset by its [`Id`] and returns it by a mutual reference if the asset exists
    pub fn get_mut<T: Asset>(&mut self, id: Id<T>) -> Option<&mut T> {
        self.map
            .get_mut(id.uuid())
            .map(|a| a.downcast_mut::<T>())
            .unwrap_or(None)
    }

    /// Removes an asset from the Service and returns it if the asset exists
    pub fn remove<T: Asset>(&mut self, id: Id<T>) -> Option<T> {
        self.map
            .remove(id.uuid())
            .map(|a| *(unsafe { Box::from_raw((Box::leak(a) as *mut dyn Asset) as *mut T) }))
    }

    /// Increments last_id counter and returns new value
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
