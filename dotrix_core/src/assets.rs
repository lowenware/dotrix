//! Assets and management service
pub mod animation;
mod load_gltf;
pub mod loader;
pub mod mesh;
pub mod resource;
pub mod shader;
pub mod skin;
pub mod texture;

pub use animation::Animation;
pub use loader::*;
pub use mesh::*;
pub use resource::*;
pub use shader::Shader;
pub use skin::Skin;
pub use texture::*;

use std::{
    any::{Any, TypeId},
    collections::{hash_map, hash_set, HashMap, HashSet},
    sync::{mpsc, Arc, Mutex},
    vec::Vec,
};

use crate::{ecs::Mut, id::Id, Renderer};

const THREADS_COUNT: usize = 4;

trait AssetSet: Any {
    fn as_any_ref(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Asset> AssetSet for HashMap<Id<T>, T> {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T: Asset> AssetSet for HashSet<Id<T>> {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Any struct can be an asset
pub trait Asset: Any + Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Asset for T {}

/// Assets management service
///
/// Stored asset can be identified by its [`Id`]. The [`Id`] is being assigned to an asset, once
/// the asset is stored.
///
/// File operations performed by the service are being executed in separate threads, so no
/// execution blocking happens. Due to some resources may contain several assets, they may not be
/// loaded at the same time. Also bigger assets may take longer time to be loaded.
///
/// There is a way to aquire asset [`Id`] immediately without awaiting using [`Assets::register`]
/// method.
pub struct Assets {
    registry: HashMap<String, u64>,
    assets: HashMap<TypeId, Box<dyn AssetSet>>,
    loaders: Vec<Loader>,
    sender: mpsc::Sender<Request>,
    receiver: mpsc::Receiver<Response>,
    id_generator: u64,
    removed_assets: HashMap<TypeId, Box<dyn AssetSet>>,
    hot_reload: bool,
    root: std::path::PathBuf,
}

impl Assets {
    /// Creates new instance of Assets container
    pub fn new() -> Self {
        let threads_count = THREADS_COUNT;
        let (sender, thread_rx) = mpsc::channel();
        let (thread_tx, receiver) = mpsc::channel();
        let thread_rx = Arc::new(Mutex::new(thread_rx));
        let thread_tx = Arc::new(Mutex::new(thread_tx));
        let mut loaders = Vec::with_capacity(threads_count);

        for id in 0..threads_count {
            loaders.push(Loader::new(
                id,
                Arc::clone(&thread_rx),
                Arc::clone(&thread_tx),
            ));
        }

        let root = std::env::current_dir().expect("Current working directory must be accessible");

        Self {
            registry: HashMap::new(),
            assets: HashMap::new(),
            loaders,
            sender,
            receiver,
            id_generator: 1,
            removed_assets: HashMap::new(),
            hot_reload: true,
            root,
        }
    }

    /// Set assets root directory
    pub fn set_root(&mut self, root: std::path::PathBuf) {
        self.root = root;
    }

    /// Get assets root directory
    pub fn root(&self) -> &std::path::Path {
        self.root.as_path()
    }

    /// Imports an asset file by its relative path and returns [`Id`] of the [`Resource`]
    pub fn import(&mut self, path_str: &str) -> Id<Resource> {
        let path = self.root.as_path().join(path_str);
        self.import_from(path)
    }

    /// Imports an asset file from specified absolute or relative path and returns [`Id`] of the
    /// [`Resource`]
    pub fn import_from(&mut self, path: std::path::PathBuf) -> Id<Resource> {
        let name = path
            .file_stem()
            .map(|n| n.to_str().unwrap())
            .unwrap()
            .to_string();
        let resource = Resource::new(name.clone(), path.as_path().display().to_string());
        let id = self.store_as::<Resource>(resource, &name);

        let task = Task { path, name };
        self.sender.send(Request::Import(task)).unwrap();
        id
    }

    /// Associates an asset name with [`Id`] and returns it
    ///
    /// If name was already used, then no changes will be done and already associated [`Id`] will
    /// be returned.
    ///
    /// As it was said, names of assets loaded with [`Assets::import`] method can be predictable
    /// in most cases. Using that prediction, developer can obtain an [`Id`] even before calling
    /// the [`Assets::import`] method.
    ///
    /// ```no_run
    /// use dotrix_core::{
    ///     assets::Texture,
    ///     ecs::Mut,
    ///     Assets,
    /// };
    ///
    /// fn my_system(mut assets: Mut<Assets>) {
    ///     // get the id
    ///     let texture = assets.register::<Texture>("my_texture");
    ///     // import the texture
    ///     assets.import("/path/to/my_texture.png");
    /// }
    /// ```
    pub fn register<T: Asset>(&mut self, name: &str) -> Id<T>
    where
        Self: AssetMapGetter<T>,
    {
        let raw_id = self.next_id();
        Id::new(*self.registry.entry(name.to_string()).or_insert(raw_id))
    }

    /// Stores an asset under user defined name and returns [`Id`] of it
    pub fn store_as<T: Asset>(&mut self, asset: T, name: &str) -> Id<T>
    where
        Self: AssetMapGetter<T>,
    {
        let id = self.register(name);
        self.map_mut().insert(id, asset);
        id
    }

    /// Stores an asset and returns [`Id`] of it
    pub fn store<T: Asset>(&mut self, asset: T) -> Id<T>
    where
        Self: AssetMapGetter<T>,
    {
        let id = Id::new(self.next_id());
        self.map_mut().insert(id, asset);
        id
    }

    /// Searches for an asset by the name and return [`Id`] of it if the asset exists
    pub fn find<T: Asset>(&self, name: &str) -> Option<Id<T>>
    where
        Self: AssetMapGetter<T>,
    {
        self.registry.get(&name.to_string()).map(|id| Id::new(*id))
    }

    /// Searches an asset by its [`Id`] and returns it by a reference if the asset exists
    pub fn get<T: Asset>(&self, handle: Id<T>) -> Option<&T>
    where
        Self: AssetMapGetter<T>,
    {
        self.map()?.get(&handle)
    }

    /// Searches an asset by its [`Id`] and returns it by a mutual reference if the asset exists
    pub fn get_mut<T: Asset>(&mut self, handle: Id<T>) -> Option<&mut T>
    where
        Self: AssetMapGetter<T>,
    {
        self.map_mut().get_mut(&handle)
    }

    /// Removes an asset from the Service and returns it if the asset exists
    pub fn remove<T: Asset>(&mut self, handle: Id<T>) -> Option<T>
    where
        Self: AssetMapGetter<T>,
    {
        self.map_removed_mut().insert(handle);
        self.map_mut().remove(&handle)
    }

    /// Returns iterator over assets by its type
    ///
    /// Must be mutable as it will create the hashmap if it does not exist
    pub fn iter<T: Asset>(&mut self) -> hash_map::Iter<'_, Id<T>, T>
    where
        Self: AssetMapGetter<T>,
    {
        self.map_mut().iter()
    }

    /// Returns mutable iterator over assets by its type
    pub fn iter_mut<T: Asset>(&mut self) -> hash_map::IterMut<'_, Id<T>, T>
    where
        Self: AssetMapGetter<T>,
    {
        self.map_mut().iter_mut()
    }

    /// Returns an iter to the removed asset list
    pub fn iter_removed<T: Asset>(&mut self) -> hash_set::Iter<'_, Id<T>>
    where
        Self: AssetMapGetter<T>,
    {
        self.map_removed_mut().iter()
    }

    /// Returns the removed asset list
    pub fn get_removed_ref<T: Asset>(&mut self) -> Option<&HashSet<Id<T>>>
    where
        Self: AssetMapGetter<T>,
    {
        self.map_removed()
    }

    /// Returns the mutable removed asset list
    pub fn get_removed_mut<T: Asset>(&mut self) -> &mut HashSet<Id<T>>
    where
        Self: AssetMapGetter<T>,
    {
        self.map_removed_mut()
    }

    fn next_id(&mut self) -> u64 {
        let result = self.id_generator;
        self.id_generator += 1;
        result
    }

    pub(crate) fn fetch(&mut self) {
        while let Ok(response) = self.receiver.try_recv() {
            match response {
                Response::Animation(animation) => {
                    self.store_as(*animation.asset, &animation.name);
                }
                Response::Mesh(mesh) => {
                    self.store_as(*mesh.asset, &mesh.name);
                }
                Response::Shader(shader) => {
                    self.store_as(*shader.asset, &shader.name);
                }
                Response::Skin(skin) => {
                    self.store_as(*skin.asset, &skin.name);
                }
                Response::Texture(texture) => {
                    self.store_as(*texture.asset, &texture.name);
                }
            };
        }
    }

    /// Enable/Disable hot reload of certain assets. If this is
    /// disabled certain assets like `Shaders` need to be
    /// cleaned up manually with `renderer.drop_pipeline`
    pub fn hot_reload_enable(&mut self, enable: bool) {
        self.hot_reload = enable;
    }
}

/// Reload assets and cleanup any assets that need some post process
/// after `assets.remove`
pub fn release(mut assets: Mut<Assets>, mut renderer: Mut<Renderer>) {
    if !assets.hot_reload {
        return;
    }
    // Shaders that no longer exist need to be removed from the renderer
    // by dropping their pipeline
    for removed_shader in assets
        .iter_removed::<Shader>()
        .copied()
        .collect::<Vec<Id<Shader>>>()
    {
        if assets.get::<Shader>(removed_shader).is_none() {
            renderer.drop_pipeline(removed_shader);
        }
    }
    // assets.get_removed_mut::<Shader>().clear();
    // TODO: Find a better way to cleanup deleted assets
    assets.removed_assets.clear();

    // Any shader that has its shader module dropped/uninitalised should have it's
    // pipeline dropped from the renderer
    for (id, shader) in assets.iter::<Shader>() {
        if !shader.loaded() {
            renderer.drop_pipeline(*id);
        }
    }
}

/// Asset map getting trait
pub trait AssetMapGetter<T> {
    /// Returns HashMap reference for selected asset type
    ///
    /// Returns an option as the map may not have been made yet
    /// and we don't have the mutable access required to make it
    fn map(&self) -> Option<&HashMap<Id<T>, T>>;
    /// Returns mutable HashMap reference for selected asset type
    fn map_mut(&mut self) -> &mut HashMap<Id<T>, T>;

    /// Returns HashSet reference for selected asset type that have been removed
    /// since last clean up
    fn map_removed(&self) -> Option<&HashSet<Id<T>>>;
    /// Returns mutable HashSet reference for selected asset type that have been removed
    /// since last clean up
    fn map_removed_mut(&mut self) -> &mut HashSet<Id<T>>;
}

impl<T: Asset> AssetMapGetter<T> for Assets {
    fn map(&self) -> Option<&HashMap<Id<T>, T>> {
        self.assets
            .get(&TypeId::of::<T>())?
            .as_any_ref()
            .downcast_ref::<HashMap<Id<T>, T>>()
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<T>, T> {
        self.assets
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                let empty: HashMap<Id<T>, T> = Default::default();
                Box::new(empty)
            })
            .as_any_mut()
            .downcast_mut::<HashMap<Id<T>, T>>()
            .unwrap()
    }
    fn map_removed(&self) -> Option<&HashSet<Id<T>>> {
        self.removed_assets
            .get(&TypeId::of::<T>())?
            .as_any_ref()
            .downcast_ref::<HashSet<Id<T>>>()
    }
    fn map_removed_mut(&mut self) -> &mut HashSet<Id<T>> {
        self.removed_assets
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                let empty: HashSet<Id<T>> = Default::default();
                Box::new(empty)
            })
            .as_any_mut()
            .downcast_mut::<HashSet<Id<T>>>()
            .unwrap()
    }
}

impl Drop for Assets {
    fn drop(&mut self) {
        for _ in &self.loaders {
            self.sender.send(Request::Terminate).unwrap();
        }
        for loader in &mut self.loaders {
            loader.join();
        }
    }
}

unsafe impl Sync for Assets {}
unsafe impl Send for Assets {}

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}
