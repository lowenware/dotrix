use std::{
    collections::HashMap,
    sync::{Arc, mpsc, Mutex},
    vec::Vec,
};
use super::{
    id::{Id, RawId},
    loader::{Loader, Request, Task},
    mesh::Mesh,
    resource::Resource,
    texture::Texture,
};

const THREADS_COUNT: usize = 4;

pub struct Service {
    registry: HashMap<String, RawId>,
    resources: HashMap<Id<Resource>, Resource>,
    textures: HashMap<Id<Texture>, Texture>,
    meshes: HashMap<Id<Mesh>, Mesh>,
    loaders: Vec<Loader>,
    sender: mpsc::Sender<Request>,
    id_generator: RawId,
}

impl Service {

    /// Creates new instance of Service container
    pub fn new() -> Self {
        let threads_count = THREADS_COUNT;
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut loaders = Vec::with_capacity(threads_count);

        for id in 0..threads_count {
            loaders.push(Loader::new(id, Arc::clone(&receiver)));
        }

        Self {
            registry: HashMap::new(),
            resources: HashMap::new(),
            textures: HashMap::new(),
            meshes: HashMap::new(),
            loaders,
            sender,
            id_generator: 1,
        }
    }

    /// imports an asset file to the container
    pub fn import(&mut self, path: &str, name: &str) -> Id<Resource> {
        let resource = Resource::new(name.to_string(), path.to_string());
        let id = self.register::<Resource>(resource, name);
        // TODO: start loading in separate thread
        let task = Box::new(Task{path: path.to_string()});
        self.sender.send(Request::Import(task)).unwrap();
        id
    }

    /// Registers new asset in the system under user defined name
    pub fn register<T>(&mut self, asset: T, name: &str) -> Id<T>
    where Self: AssetMapGetter<T> {
        let raw_id = self.next_id();
        let id = Id::new(raw_id);
        self.registry.insert(name.to_string(), raw_id);
        self.map_mut().insert(id, asset);
        id
    }

    /// Finds an asset Id by its name
    pub fn find<T>(&self, name: &str) -> Option<Id<T>>
    where Self: AssetMapGetter<T> {
        self.registry.get(&name.to_string()).map(|raw_id| Id::new(*raw_id))
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
}

pub trait AssetMapGetter<T> {
    fn map(&self) -> &HashMap<Id<T>, T>;
    fn map_mut(&mut self) -> &mut HashMap<Id<T>, T>;
}

impl AssetMapGetter<Texture> for Service {
    fn map(&self) -> &HashMap<Id<Texture>, Texture> {
        &self.textures
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Texture>, Texture> {
        &mut self.textures
    }
}

impl AssetMapGetter<Mesh> for Service {
    fn map(&self) -> &HashMap<Id<Mesh>, Mesh> {
        &self.meshes
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Mesh>, Mesh> {
        &mut self.meshes
    }
}

impl AssetMapGetter<Resource> for Service {
    fn map(&self) -> &HashMap<Id<Resource>, Resource> {
        &self.resources
    }

    fn map_mut(&mut self) -> &mut HashMap<Id<Resource>, Resource> {
        &mut self.resources
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        for _ in &self.loaders {
            self.sender.send(Request::Terminate).unwrap();
        }
        for loader in &mut self.loaders {
            loader.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
