use std::{
    any::TypeId,
    collections::HashMap,
};

use super::Service;

pub struct Services {
    storage: HashMap<TypeId, Box<dyn std::any::Any>>,
}

impl Services {

    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn add<T: Service>(&mut self, service: T) {
        self.storage.insert(TypeId::of::<T>(), Box::new(service));
    }

    pub fn get<T: Service>(&self) -> Option<&T> {
        self.storage
            .get(&TypeId::of::<T>())
            .map(|srv| srv.downcast_ref::<T>().unwrap())
    }

    pub fn get_mut<T: Service>(&mut self) -> Option<&mut T> {
        self.storage
            .get_mut(&TypeId::of::<T>())
            .map(|srv| srv.downcast_mut::<T>().unwrap())
    }
}

impl Default for Services {
    fn default() -> Self {
        Self::new()
    }
}
