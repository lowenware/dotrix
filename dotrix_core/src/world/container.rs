use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use super::Archetype;
use crate::ecs::Component;

pub struct Container {
    components: HashMap<TypeId, Box<dyn Stripe>>,
}

trait Stripe: Any {
    fn remove_by_index(&mut self, index: usize);
    fn as_any_ref(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> Stripe for Vec<T> {
    fn remove_by_index(&mut self, index: usize) {
        self.remove(index);
    }
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
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
        if let Some(v) = self.components.get_mut(&TypeId::of::<T>()) {
            v.as_any_mut()
                .downcast_mut::<Vec<T>>()
                .unwrap()
                .push(component)
        }
    }

    pub fn init<T: Component>(&mut self) {
        self.components
            .insert(TypeId::of::<T>(), Box::new(Vec::<T>::new()));
    }

    pub fn get_mut<T: Component>(&self) -> Option<&mut Vec<T>>
    where
        T: Component,
    {
        self.components.get(&TypeId::of::<T>()).map(|v| unsafe {
            let vec_ref = v.as_any_ref().downcast_ref::<Vec<T>>().unwrap();
            let vec_ptr = vec_ref as *const Vec<T>;
            let mut_ptr = vec_ptr as *mut Vec<T>;
            &mut *mut_ptr
        })
    }

    pub fn get<T: Component>(&self) -> Option<&Vec<T>>
    where
        T: Component,
    {
        self.components
            .get(&TypeId::of::<T>())
            .map(|v| v.as_any_ref().downcast_ref::<Vec<T>>().unwrap())
    }

    pub fn has(&self, key: TypeId) -> bool {
        self.components.contains_key(&key)
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn remove(&mut self, index: usize) {
        for stripe in self.components.values_mut() {
            stripe.remove_by_index(index);
        }
    }
}

#[cfg(test)]
mod tests {
    struct Item1(u32);
    struct Item2(u32);
    use crate::world::container::Container;
    #[test]
    fn mutability() {
        let mut c = Container::new::<(Item1, Item2)>();
        c.push::<Item1>(Item1(123));
        c.push::<Item2>(Item2(666));

        for i in c.get_mut::<Item1>().unwrap() {
            i.0 += 198;
        }

        for i in c.get::<Item1>().unwrap() {
            assert_eq!(i.0, 321);
        }
    }
}
