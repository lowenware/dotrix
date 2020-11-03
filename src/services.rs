use std::{
    any::TypeId,
    collections::HashMap,
};

pub struct Services {
    storage: HashMap<TypeId, Box<dyn std::any::Any>>,
}

pub trait Service: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Service for T {}

impl Services {

    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn add<T: Service>(&mut self, service: T) {
        self.storage.insert(TypeId::of::<T>(), Box::new(service));
    }

    pub fn get<T: Service>(&self) -> &T
    {
        self.storage.get(&TypeId::of::<T>()).unwrap().downcast_ref::<T>().unwrap()
    }

    pub fn get_mut<T: Service>(&mut self) -> &mut T
    {
        self.storage.get_mut(&TypeId::of::<T>()).unwrap().downcast_mut::<T>().unwrap()
    }

}
