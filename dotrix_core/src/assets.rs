//! Assets and management service
pub mod animation;
pub mod loader;
mod load_gltf;
pub mod mesh;
pub mod shader;
pub mod skin;
pub mod resource;
pub mod texture;

pub use loader::*;
pub use animation::Animation;
pub use mesh::*;
pub use shader::Shader;
pub use skin::Skin;
pub use resource::*;
pub use texture::*;

use std::{
    collections::{hash_map, HashMap, HashSet},
    sync::{mpsc, Arc, Mutex},
    vec::Vec,
};

use crate::{ecs::Mut, id::Id, Renderer};

const THREADS_COUNT: usize = 4;

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
    resources: HashMap<Id<Resource>, Resource>,
    animations: HashMap<Id<Animation>, Animation>,
    textures: HashMap<Id<Texture>, Texture>,
    meshes: HashMap<Id<Mesh>, Mesh>,
    shaders: HashMap<Id<Shader>, Shader>,
    skins: HashMap<Id<Skin>, Skin>,
    loaders: Vec<Loader>,
    sender: mpsc::Sender<Request>,
    receiver: mpsc::Receiver<Response>,
    id_generator: u64,
    removed_shaders: HashSet<Id<Shader>>,
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
            loaders.push(Loader::new(id, Arc::clone(&thread_rx), Arc::clone(&thread_tx)));
        }

        Self {
            registry: HashMap::new(),
            resources: HashMap::new(),
            animations: HashMap::new(),
            textures: HashMap::new(),
            meshes: HashMap::new(),
            shaders: HashMap::new(),
            skins: HashMap::new(),
            loaders,
            sender,
            receiver,
            id_generator: 1,
            removed_shaders: HashSet::new(),
        }
    }

    /// Imports an asset file by its path and returns [`Id`] of the [`Resource`]
    pub fn import(&mut self, path_str: &str) -> Id<Resource> {
        let path = std::path::Path::new(path_str);
        let name = path.file_stem().map(|n| n.to_str().unwrap()).unwrap();
        let resource = Resource::new(name.to_string(), path_str.to_string());
        let id = self.store_as::<Resource>(resource, name);
        // TODO: start loading in separate thread
        let task = Task { path: path.to_path_buf(), name: name.to_string() };
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
    ///     services::Assets,
    /// };
    ///
    /// fn my_system(mut assets: Mut<Assets>) {
    ///     // get the id
    ///     let texture = assets.register::<Texture>("my_texture");
    ///     // import the texture
    ///     assets.import("/path/to/my_texture.png");
    /// }
    /// ```
    pub fn register<T>(&mut self, name: &str) -> Id<T>
    where Self: AssetMapGetter<T> {
        let raw_id = self.next_id();
        Id::new(*self.registry.entry(name.to_string()).or_insert(raw_id))
    }

    /// Stores an asset under user defined name and returns [`Id`] of it
    pub fn store_as<T>(&mut self, asset: T, name: &str) -> Id<T>
    where Self: AssetMapGetter<T> {
        let id = self.register(name);
        self.map_mut().insert(id, asset);
        id
    }

    /// Stores an asset and returns [`Id`] of it
    pub fn store<T>(&mut self, asset: T) -> Id<T>
    where Self: AssetMapGetter<T> {
        let id = Id::new(self.next_id());
        self.map_mut().insert(id, asset);
        id
    }

    /// Searches for an asset by the name and return [`Id`] of it if the asset exists
    pub fn find<T>(&self, name: &str) -> Option<Id<T>>
    where Self: AssetMapGetter<T> {
        self.registry.get(&name.to_string()).map(|id| Id::new(*id))
    }

    /// Searches an asset by its [`Id`] and returns it by a reference if the asset exists
    pub fn get<T>(&self, handle: Id<T>) -> Option<&T>
    where Self: AssetMapGetter<T> {
        self.map().get(&handle)
    }

    /// Searches an asset by its [`Id`] and returns it by a mutual reference if the asset exists
    pub fn get_mut<T>(&mut self, handle: Id<T>) -> Option<&mut T>
    where Self: AssetMapGetter<T> {
        self.map_mut().get_mut(&handle)
    }

    /// Removes an asset from the Service and returns it if the asset exists
    pub fn remove<T>(&mut self, handle: Id<T>) -> Option<T>
    where Self: AssetMapGetter<T> {
        self.map_removed_mut().map(|l| l.insert(handle));
        self.map_mut().remove(&handle)
    }

    /// Returns iterator over assets by its type
    pub fn iter<T>(&mut self) -> hash_map::Iter<'_, Id<T>, T>
    where Self: AssetMapGetter<T> {
        self.map().iter()
    }

    /// Returns mutable iterator over assets by its type
    pub fn iter_mut<T>(&mut self) -> hash_map::IterMut<'_, Id<T>, T>
    where Self: AssetMapGetter<T> {
        self.map_mut().iter_mut()
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
                },
                Response::Mesh(mesh) => {
                    self.store_as(*mesh.asset, &mesh.name);
                },
                Response::Shader(shader) => {
                    self.store_as(*shader.asset, &shader.name);
                },
                Response::Skin(skin) => {
                    self.store_as(*skin.asset, &skin.name);
                },
                Response::Texture(texture) => {
                    self.store_as(*texture.asset, &texture.name);
                },
            };
        }
    }
}

/// Reload assets and cleanup any assets that need some post process
/// after `assets.remove`
pub fn assets_reload(mut assets: Mut<Assets>, mut renderer: Mut<Renderer>) {
    // Shaders that no longer exist need ro be removed from the renderer
    // by dropping their pipeline
    for removed_shader in &assets.removed_shaders {
        if assets.get::<Shader>(*removed_shader).is_none() {
            renderer.drop_pipeline(*removed_shader);
        }
    }
    assets.removed_shaders.clear();

    // Any shader that has its shader module dropped/uninitalised should have it's
    // pipeline dropped from the renderer
    for (id, shader) in &assets.shaders {
        if !shader.loaded() {
            renderer.drop_pipeline(*id);
        }
    }
}

/// Asset map getting trait
pub trait AssetMapGetter<T> {
    /// Returns HashMap reference for selected asset type
    fn map(&self) -> &HashMap<Id<T>, T>;
    /// Returns mutable HashMap reference for selected asset type
    fn map_mut(&mut self) -> &mut HashMap<Id<T>, T>;

    /// Returns HashSet reference for selected asset type that have been removed
    /// since last clean up
    fn map_removed(&self) -> Option<&HashSet<Id<T>>> {
        None
    }
    /// Returns mutable HashSet reference for selected asset type that have been removed
    /// since last clean up
    fn map_removed_mut(&mut self) -> Option<&mut HashSet<Id<T>>> {
        None
    }
}

impl AssetMapGetter<Animation> for Assets {
    fn map(&self) -> &HashMap<Id<Animation>, Animation> {
        &self.animations
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Animation>, Animation> {
        &mut self.animations
    }
}

impl AssetMapGetter<Texture> for Assets {
    fn map(&self) -> &HashMap<Id<Texture>, Texture> {
        &self.textures
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Texture>, Texture> {
        &mut self.textures
    }
}

impl AssetMapGetter<Mesh> for Assets {
    fn map(&self) -> &HashMap<Id<Mesh>, Mesh> {
        &self.meshes
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Mesh>, Mesh> {
        &mut self.meshes
    }
}

impl AssetMapGetter<Skin> for Assets {
    fn map(&self) -> &HashMap<Id<Skin>, Skin> {
        &self.skins
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Skin>, Skin> {
        &mut self.skins
    }
}

impl AssetMapGetter<Resource> for Assets {
    fn map(&self) -> &HashMap<Id<Resource>, Resource> {
        &self.resources
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Resource>, Resource> {
        &mut self.resources
    }
}

impl AssetMapGetter<Shader> for Assets {
    fn map(&self) -> &HashMap<Id<Shader>, Shader> {
        &self.shaders
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Shader>, Shader> {
        &mut self.shaders
    }

    fn map_removed(&self) -> Option<&HashSet<Id<Shader>>> {
        Some(&self.removed_shaders)
    }
    fn map_removed_mut(&mut self) -> Option<&mut HashSet<Id<Shader>>> {
        Some(&mut self.removed_shaders)
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
