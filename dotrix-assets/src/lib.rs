mod asset;
mod loader;
mod tasks;

use std::collections::HashMap;

use dotrix_core as dotrix;
use dotrix_types::Id;

pub use asset::{Asset, Bundle, File, Resource};
use core::ops::{Deref, DerefMut};
pub use loader::{LoadError, Loader};
pub use tasks::{LoadTask, StoreTask, Watchdog};

pub const NAMESPACE: u64 = dotrix::NAMESPACE | 0x0100;

// TODO: update asset version on rewrite
struct Slot {
    name: String,
    version: u32,
    asset: Box<dyn Asset>,
}

impl Slot {
    pub fn new(asset: Box<dyn Asset>) -> Self {
        Self::named(asset, Default::default())
    }

    pub fn named(asset: Box<dyn Asset>, name: String) -> Self {
        Self {
            name,
            version: 0,
            asset,
        }
    }

    pub fn link<T: Asset>(&self) -> Option<Link<&T>> {
        let version = self.version;
        let name = &self.name;
        self.asset.downcast_ref::<T>().map(|asset| Link {
            name,
            version,
            asset,
        })
    }

    pub fn link_mut<T: Asset>(&mut self) -> Option<Link<&mut T>> {
        let version = self.version;
        let name = &self.name;
        self.asset.downcast_mut::<T>().map(|asset| Link {
            name,
            version,
            asset,
        })
    }
}

pub struct Link<'a, T> {
    name: &'a str,
    version: u32,
    asset: T,
}

impl<'a, T> Link<'a, T> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

impl<'a, T> Deref for Link<'a, &'a T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.asset
    }
}

impl<'a, T> Deref for Link<'a, &'a mut T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.asset
    }
}

impl<'a, T> DerefMut for Link<'a, &'a mut T> {
    fn deref_mut(&mut self) -> &mut T {
        self.asset
    }
}

// TODO: update asset version on rewrite

/// Assets management service
pub struct Assets {
    /// Assets root folder path
    root: std::path::PathBuf,
    /// Index of IDs assigned by asset name
    registry: HashMap<String, uuid::Uuid>,
    /// Id indexed assets map
    map: HashMap<uuid::Uuid, Slot>,
    /// Resources registry indexed by file path
    resources: HashMap<std::path::PathBuf, Resource>,
    /// Asset Loaders
    loaders: HashMap<std::any::TypeId, Box<dyn Loader>>,
    /// Id counter
    last_id: u64,
}

impl Assets {
    /// Constructs new [`Assets`] instance
    pub fn new(root: &std::path::Path) -> Self {
        let root = if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()
                .expect("Current working directory must be accessible")
                .join(root)
        };

        Self {
            root,
            registry: HashMap::new(),
            map: HashMap::new(),
            resources: HashMap::new(),
            loaders: HashMap::new(),
            last_id: 0,
        }
    }

    /// Installs new asset loader
    pub fn install_raw(&mut self, loader_type_id: std::any::TypeId, loader: Box<dyn Loader>) {
        self.loaders.insert(loader_type_id, loader);
    }

    /// Installs new asset loader
    pub fn install<T: Loader>(&mut self, loader: T) {
        self.install_raw(std::any::TypeId::of::<T>(), Box::new(loader));
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
        self.map
            .insert(*id.uuid(), Slot::named(Box::new(asset), String::from(name)));
        id
    }

    /// Stores an asset and returns [`Id`] of it
    pub fn store<T: Asset>(&mut self, asset: T) -> Id<T> {
        let id = T::id(self.next_id());
        self.map.insert(*id.uuid(), Slot::new(Box::new(asset)));
        id
    }

    /// Stores an asset under user defined name and returns [`Id`] of it
    pub(crate) fn store_raw(&mut self, asset: Box<dyn Asset>) {
        let uuid = self
            .registry
            .get(asset.name())
            .map(|id| *id)
            .unwrap_or_else(|| uuid::Uuid::from_u64_pair(asset.namespace(), self.next_id()));

        self.map.insert(uuid, Slot::new(asset));
    }

    /// Searches for an asset by the name and return [`Id`] of it if the asset exists
    pub fn find<T: Asset>(&self, name: &str) -> Option<Id<T>> {
        self.registry.get(name).map(|uuid| Id::from(*uuid))
    }

    /// Searches an asset by its [`Id`] and returns it by a reference if the asset exists
    pub fn get<T: Asset>(&self, id: Id<T>) -> Option<Link<&T>> {
        self.map.get(id.uuid()).and_then(|a| a.link::<T>())
    }

    /// Searches an asset by its [`Id`] and returns it by a mutual reference if the asset exists
    pub fn get_mut<T: Asset>(&mut self, id: Id<T>) -> Option<Link<&mut T>> {
        self.map.get_mut(id.uuid()).and_then(|a| a.link_mut::<T>())
    }

    /// Removes an asset from the Service and returns it if the asset exists
    pub fn remove<T: Asset>(&mut self, id: Id<T>) -> Option<T> {
        self.map.remove(id.uuid()).map(|slot| {
            *(unsafe { Box::from_raw((Box::leak(slot.asset) as *mut dyn Asset) as *mut T) })
        })
    }

    pub(crate) fn resource(&mut self, path: std::path::PathBuf) -> &mut Resource {
        self.resources
            .entry(path.clone())
            .or_insert_with(|| Resource::new(path))
    }

    pub(crate) fn resources(
        &self,
    ) -> std::collections::hash_map::Values<std::path::PathBuf, Resource> {
        self.resources.values()
    }

    pub(crate) fn loaders(
        &self,
    ) -> std::collections::hash_map::Values<std::any::TypeId, Box<dyn Loader>> {
        self.loaders.values()
    }

    /// Increments last_id counter and returns new value
    fn next_id(&mut self) -> u64 {
        self.last_id += 1;
        self.last_id
    }
}

pub struct Extension {
    pub root: std::path::PathBuf,
    pub hot_reload: bool,
    pub init: fn(&mut Assets),
}

fn dummy_init(_: &mut Assets) {}

impl Default for Extension {
    fn default() -> Self {
        Self {
            root: std::path::PathBuf::from(r"./"),
            hot_reload: false,
            init: dummy_init,
        }
    }
}

impl dotrix::Extension for Extension {
    fn add_to(&self, manager: &mut dotrix::Manager) {
        let mut assets = Assets::new(&self.root);
        (self.init)(&mut assets);

        manager.store(assets);
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
        fn can_load(&self, _: &std::path::Path) -> bool {
            false
        }

        fn load(&self, path: &std::path::Path, data: Vec<u8>) -> Vec<Box<dyn Asset>> {
            vec![]
        }
    }

    #[test]
    fn install_uninstall_loader() {
        let control_value = 57651236;
        let mut assets = Assets::new(std::path::Path::new("./"));
        assets.install(DummyLoader(control_value));
        let loader: Option<DummyLoader> = assets.uninstall();
        assert_eq!(loader.is_some(), true);
        assert_eq!(loader.unwrap().0, control_value);
    }
}
