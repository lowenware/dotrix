mod id;
mod loader;
mod mesh;
mod resource;
mod texture;

pub use id::*;
pub use loader::*;
pub use mesh::*;
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
    textures: HashMap<Id<Texture>, Texture>,
    meshes: HashMap<Id<Mesh>, Mesh>,
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
            textures: HashMap::new(),
            meshes: HashMap::new(),
            loaders,
            sender,
            receiver,
            id_generator: 1,
        }
    }

    /// imports an asset file to the container
    pub fn import(&mut self, path: &str, name: &str) -> Id<Resource> {
        let resource = Resource::new(name.to_string(), path.to_string());
        let id = self.register::<Resource>(resource, name.to_string());
        // TODO: start loading in separate thread
        let task = Task { path: path.to_string(), name: name.to_string() };
        self.sender.send(Request::Import(task)).unwrap();
        id
    }

    /// Registers new asset in the system under user defined name
    pub fn register<T>(&mut self, asset: T, name: String) -> Id<T>
    where Self: AssetMapGetter<T> {
        let raw_id = self.next_id();
        let id = Id::new(raw_id);
        self.registry.insert(name, raw_id);
        self.map_mut().insert(id, asset);
        id
    }

    /// Finds an asset Id by its name
    pub fn find<T>(&mut self, name: &str) -> Id<T>
    where Self: AssetMapGetter<T> {
        let raw_id = self.next_id();
        Id::new(*self.registry.entry(name.to_string()).or_insert(raw_id))
    }

    /// Gets an asset by the handle
    pub fn get<T>(&self, handle: Id<T>) -> Option<&T>
    where Self: AssetMapGetter<T> {
        self.map().get(&handle)
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
                Response::Texture(texture) => {
                    let id = self.find::<Texture>(texture.name.as_str());
                    self.map_mut().insert(id, texture.asset);
                },
                Response::Mesh(mesh) => {
                    let id = self.find::<Mesh>(mesh.name.as_str());
                    self.map_mut().insert(id, mesh.asset);
                },
            };
        }
    }
}

pub trait AssetMapGetter<T> {
    fn map(&self) -> &HashMap<Id<T>, T>;
    fn map_mut(&mut self) -> &mut HashMap<Id<T>, T>;
}

impl AssetMapGetter<Texture> for Assets {
    fn map(&self) -> &HashMap<Id<Texture>, Texture> {
        &self.textures
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Texture>, Texture> {
        &mut self.textures
        // &mut self.storage.lock().unwrap().textures
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
