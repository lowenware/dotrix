use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{
    ecs::Component,
    world::Archetype,
};

pub struct Container {
    components: HashMap<TypeId, Box<dyn Any>>,
}

impl Container {
    pub fn new<A: Archetype>() -> Self {
        let mut result = Self {
            components: HashMap::new(),
        };
        A::map(&mut result);
        result
    }

    pub fn push<T: Component>(&mut self, component: T) {
        self.components
            .get_mut(&TypeId::of::<T>())
            .map(|v| {
                v.downcast_mut::<Vec<T>>()
                    .unwrap()
                    .push(component)
            });
    }

    pub fn init<T: Component>(&mut self) {
        self.components.insert(TypeId::of::<T>(), Box::new(Vec::<T>::new()));
    }

    pub fn get_mut<T: Component>(&mut self) -> Option<&mut Vec<T>>
    where T: Component
    {
        self.components
            .get_mut(&TypeId::of::<T>())
            .map(|v| v.downcast_mut::<Vec<T>>().unwrap())
    }

    pub fn get<T: Component>(&self) -> Option<&Vec<T>>
    where T: Component
    {
        self.components
            .get(&TypeId::of::<T>())
            .map(|v| v.downcast_ref::<Vec<T>>().unwrap())
    }

    pub fn has(&self, key: TypeId) -> bool {
        self.components.get(&key).is_some()
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }
}

