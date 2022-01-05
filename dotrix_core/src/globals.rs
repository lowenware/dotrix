use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Centralized container service for uniforms and other data
#[derive(Default)]
pub struct Globals {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Globals {
    /// Gets a reference to the data instance by its type
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.map
            .get(&type_id)
            .map(|boxed| boxed.downcast_ref::<T>().unwrap())
    }

    /// Gets a mutable reference to the data instance by its type
    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.map
            .get_mut(&type_id)
            .map(|boxed| boxed.downcast_mut::<T>().unwrap())
    }

    /// Stores the data instance by its type
    pub fn set<T: 'static + Send + Sync>(&mut self, entry: T) {
        let type_id = TypeId::of::<T>();
        self.map.insert(type_id, Box::new(entry));
    }

    /// Removes the data by its type
    pub fn remove<T: 'static + Send + Sync>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.map.remove(&type_id);
    }
}
