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

    pub fn get_mut<T: Component>(&self) -> Option<&mut Vec<T>>
    where T: Component
    {
        self.components
            .get(&TypeId::of::<T>())
            .map(|v| {
                unsafe {
                    let vec_ref = v.downcast_ref::<Vec<T>>().unwrap();
                    let vec_ptr = vec_ref as *const Vec<T>;
                    let mut_ptr = vec_ptr as *mut Vec<T>;
                    &mut *mut_ptr
                }
            })
    }

    pub fn get<T: Component>(&self) -> Option<&Vec<T>>
    where T: Component
    {
        self.components
            .get(&TypeId::of::<T>())
            .map(|v| v.downcast_ref::<Vec<T>>().unwrap())
    }

    /*
    pub fn select<A, B, C>(&mut self) -> (std::slice::Iter<A>, std::slice::IterMut<B>, std::slice::IterMut<C>)
    where
        A: Component,
        B: Component,
        C: Component,
    {
        self.components.into_iter()
            .filter(|k| k.0 == TypeId::of::<A>() || k.0 == TypeId::of::<C>() || k.0 == TypeId::of::<B>())
            .map(|v| match v.0 {
                TypeId::of::<A>() => v.1.downcast_mut::<Vec<A>>().unwrap(),
                TypeId::of::<B>() => v.1.downcast_mut::<Vec<B>>().unwrap(),
                TypeId::of::<C>() => v.1.downcast_mut::<Vec<C>>().unwrap(),
            })
            .flatten()
    }
    */

    pub fn has(&self, key: TypeId) -> bool {
        self.components.contains_key(&key)
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }
}

#[cfg(test)]
mod tests {
    struct Item1(u32);
    struct Item2(u32);
    use crate::container::Container;
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
