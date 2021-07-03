use std::{
    any::{ Any, TypeId },
    collections::HashMap,
};

#[derive(Default)]
pub struct Globals {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Globals {
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id)
            .map(|boxed| boxed.downcast_ref::<T>().unwrap())
    }

    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.map.get_mut(&type_id)
            .map(|boxed| boxed.downcast_mut::<T>().unwrap())
    }

    pub fn set<T: 'static + Send + Sync>(&mut self, entry: T) {
        let type_id = TypeId::of::<T>();
        self.map.insert(type_id, Box::new(entry));
    }

    pub fn remove<T: 'static + Send + Sync>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.map.remove(&type_id);
    }
}
