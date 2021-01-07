mod animation;
mod id;
mod loader;
mod load_gltf;
mod mesh;
mod skin;
mod resource;
mod texture;

pub use id::*;
pub use loader::*;
pub use animation::Animation;
pub use mesh::*;
pub use skin::{Skin, Pose}; // TODO: consider moving of Pose to some shared place
pub use resource::*;
pub use texture::*;

use std::{
    collections::HashMap,
    sync::{Arc, mpsc, Mutex},
    vec::Vec,
};

const THREADS_COUNT: usize = 4;

pub struct Assets {
    registry: HashMap<String, RawId>,
    resources: HashMap<Id<Resource>, Resource>,
    animations: HashMap<Id<Animation>, Animation>,
    textures: HashMap<Id<Texture>, Texture>,
    meshes: HashMap<Id<Mesh>, Mesh>,
    skins: HashMap<Id<Skin>, Skin>,
    loaders: Vec<Loader>,
    sender: mpsc::Sender<Request>,
    receiver: mpsc::Receiver<Response>,
    id_generator: RawId,
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
            skins: HashMap::new(),
            loaders,
            sender,
            receiver,
            id_generator: 1,
        }
    }

    /// imports an asset file to the container
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

    /// stores new asset in the system under user defined name
    pub fn store_as<T>(&mut self, asset: T, name: &str) -> Id<T>
    where Self: AssetMapGetter<T> {
        let id = self.register(name);
        self.map_mut().insert(id, asset);
        id
    }

    /// stores new asset in the system under user defined name
    pub fn store<T>(&mut self, asset: T) -> Id<T>
    where Self: AssetMapGetter<T> {
        let id = Id::new(self.next_id());
        self.map_mut().insert(id, asset);
        id
    }

    /// Registers user defined name for an asset
    pub fn register<T>(&mut self, name: &str) -> Id<T>
    where Self: AssetMapGetter<T> {
        let raw_id = self.next_id();
        Id::new(*self.registry.entry(name.to_string()).or_insert(raw_id))
    }

    pub fn find<T>(&self, name: &str) -> Option<Id<T>>
    where Self: AssetMapGetter<T> {
        self.registry.get(&name.to_string()).map(|id| Id::new(*id))
    }

    /// Gets an asset by the handle
    pub fn get<T>(&self, handle: Id<T>) -> Option<&T>
    where Self: AssetMapGetter<T> {
        self.map().get(&handle)
    }

    /// Gets an asset by the handle
    pub fn get_mut<T>(&mut self, handle: Id<T>) -> Option<&mut T>
    where Self: AssetMapGetter<T> {
        self.map_mut().get_mut(&handle)
    }

    /// Remove asset by the handle
    pub fn remove<T>(&mut self, handle: Id<T>) -> Option<T>
    where Self: AssetMapGetter<T> {
        self.map_mut().remove(&handle)
    }

    /// Returns an Id for an asset and increments the internal generator
    fn next_id(&mut self) -> RawId {
        let result = self.id_generator;
        self.id_generator += 1;
        result
    }

    pub fn fetch(&mut self) {
        while let Ok(response) = self.receiver.try_recv() {
            match response {
                Response::Animation(animation) => {
                    self.store_as(*animation.asset, &animation.name);
                    //let id = self.find::<Animation>(animation.name.as_str());
                    //self.map_mut().insert(id, animation.asset);
                },
                Response::Mesh(mesh) => {
                    self.store_as(*mesh.asset, &mesh.name);
                    //let id = self.find::<Mesh>(mesh.name.as_str());
                    //self.map_mut().insert(id, mesh.asset);
                },
                Response::Skin(skin) => {
                    self.store_as(*skin.asset, &skin.name);
                    //let id = self.find::<Skin>(skin.name.as_str());
                    //self.map_mut().insert(id, skin.asset);
                },
                Response::Texture(texture) => {
                    self.store_as(*texture.asset, &texture.name);
                    //let id = self.find::<Texture>(texture.name.as_str());
                    //self.map_mut().insert(id, texture.asset);
                },
            };
        }
    }
}

pub trait AssetMapGetter<T> {
    fn map(&self) -> &HashMap<Id<T>, T>;
    fn map_mut(&mut self) -> &mut HashMap<Id<T>, T>;
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
