use std::collections::HashMap;
use super::{
    id::{Id, RawId},
    mesh::Mesh,
    resource::Resource,
    texture::Texture,
};

pub struct Service {
    registry: HashMap<String, RawId>,
    resources: HashMap<Id<Resource>, Resource>,
    textures: HashMap<Id<Texture>, Texture>,
    meshes: HashMap<Id<Mesh>, Mesh>,
    id_generator: RawId,
}

impl Service {

    /// Creates new instance of Service container
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            resources: HashMap::new(),
            textures: HashMap::new(),
            meshes: HashMap::new(),
            id_generator: 1,
        }
    }

    /// imports an asset file to the container
    // pub fn import(&mut self, path: String, name: String) -> Id<AssetFile> {
        
    // }

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
    pub fn find<T>(&self, name: &String) -> Option<Id<T>>
    where Self: AssetMapGetter<T> {
        self.registry.get(name).map(|raw_id| Id::new(*raw_id))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
