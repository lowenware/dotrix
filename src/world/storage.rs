use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::{hash_map, HashMap};
use std::marker::PhantomData;

/// Entity structure has only id field and represent an agregation of components
pub struct Entity {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl Entity {
    pub fn new<T: IntoEntity>(tuple: T) -> Self {
        tuple.entity()
    }

    pub fn empty() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn with<T: Any>(mut self, component: T) -> Self {
        self.map.insert(TypeId::of::<T>(), Box::new(component));
        self
    }

    pub fn set_raw(&mut self, component_type_id: TypeId, component: Box<dyn Any>) {
        self.map.insert(component_type_id, component);
    }

    pub fn archetype(&self) -> Archetype {
        Archetype {
            inner: self.map.keys(),
            len: self.map.len(),
        }
    }
}

pub struct Archetype<'a> {
    inner: hash_map::Keys<'a, TypeId, Box<dyn Any>>,
    len: usize,
}

impl<'a> Archetype<'a> {
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<'a> Iterator for Archetype<'a> {
    type Item = &'a TypeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl IntoIterator for Entity {
    type Item = (TypeId, Box<dyn Any>);
    type IntoIter = hash_map::IntoIter<TypeId, Box<dyn Any>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

pub trait IntoEntity {
    fn entity(self) -> Entity;
    /// Returns true if object of this type does not have fixed set of components, false otherwise
    fn volatile() -> bool;
}

impl IntoEntity for Entity {
    fn entity(self) -> Entity {
        self
    }

    fn volatile() -> bool {
        true
    }
}

macro_rules! impl_into_components_map {
    (($($i: ident),*)) => {
        impl<$($i,)*> IntoEntity for ($($i,)*)
        where
            $($i: Any,)*
        {
            #[allow(non_snake_case)]
            fn entity(self) -> Entity {
                let ($($i,)*) = self;
                let map = [
                    $((TypeId::of::<$i>(), Box::new($i) as Box<dyn Any>),)*
                ]
                .into_iter()
                .collect::<HashMap<_,_>>();
                Entity {
                    map
                }
            }
            fn volatile() -> bool {
                false
            }
        }
    }
}

impl_into_components_map!((A));
impl_into_components_map!((A, B));
impl_into_components_map!((A, B, C));
impl_into_components_map!((A, B, C, D));
impl_into_components_map!((A, B, C, D, E));
impl_into_components_map!((A, B, C, D, E, F));
impl_into_components_map!((A, B, C, D, E, F, G));
impl_into_components_map!((A, B, C, D, E, F, G, H));
impl_into_components_map!((A, B, C, D, E, F, G, H, I));
impl_into_components_map!((A, B, C, D, E, F, G, H, I, J));
impl_into_components_map!((A, B, C, D, E, F, G, H, I, J, K));
impl_into_components_map!((A, B, C, D, E, F, G, H, I, J, K, L));
impl_into_components_map!((A, B, C, D, E, F, G, H, I, J, K, L, M));
impl_into_components_map!((A, B, C, D, E, F, G, H, I, J, K, L, M, N));
impl_into_components_map!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O));
impl_into_components_map!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P));

/// Stores Entities of the same archetype
pub struct Container {
    /// TypeId identifies component
    /// Vec stores components of different entities
    data: HashMap<TypeId, Vec<Option<UnsafeCell<Box<dyn Any>>>>>,
    removed: Vec<usize>,
    len: usize,
}

impl Container {
    pub fn contains(&self, component_type_id: TypeId) -> bool {
        self.data.contains_key(&component_type_id)
    }

    pub fn matches(&self, archetype: &mut Archetype) -> bool {
        !archetype.any(|component_type_id| !self.data.contains_key(&component_type_id))
    }

    pub fn store(&mut self, entity: Entity) -> usize {
        let index = self.removed.pop().unwrap_or_else(|| self.next_index());

        for (component_type_id, component) in entity.into_iter() {
            self.data
                .get_mut(&component_type_id)
                .expect("Entity should match container")
                .insert(index, Some(UnsafeCell::new(component)));
        }

        index
    }

    pub fn remove(&mut self, index: usize) -> Entity {
        let mut entity = Entity::empty();
        for (type_id, list) in self.data.iter_mut() {
            if let Some(component) = list[index].take() {
                entity.set_raw(*type_id, component.into_inner());
            }
        }
        self.removed.push(index);
        entity
    }

    pub fn get<C: Any>(&self, entity_index: usize) -> Option<&C> {
        self.data
            .get(&TypeId::of::<C>())
            .and_then(|list| list[entity_index].as_ref())
            .map(|value| unsafe { (&mut *(value.get())).downcast_ref::<C>().unwrap() })
    }

    pub unsafe fn get_mut<C: Any>(&self, entity_index: usize) -> Option<&mut C> {
        self.data
            .get(&TypeId::of::<C>())
            .and_then(|list| list[entity_index].as_ref())
            .map(|value| (&mut (*(value.get()))).downcast_mut::<C>().unwrap())
    }

    pub fn iter<'a, C: Any>(&'a self) -> Iter<'a, C> {
        Iter {
            inner: self.data.get(&TypeId::of::<C>()).unwrap().iter(),
            _phantom_data: PhantomData,
        }
    }

    pub unsafe fn iter_mut<'a, C: Any>(&'a self) -> IterMut<'a, C> {
        IterMut {
            inner: self.data.get(&TypeId::of::<C>()).unwrap().iter(),
            _phantom_data: PhantomData,
        }
    }

    pub fn has(&self, component_type_id: TypeId) -> bool {
        self.data.contains_key(&component_type_id)
    }

    pub fn count_components(&self) -> usize {
        self.data.len()
    }

    fn next_index(&mut self) -> usize {
        let index = self.len;
        self.len += 1;
        index
    }
}

pub struct Iter<'a, C> {
    inner: std::slice::Iter<'a, Option<UnsafeCell<Box<dyn Any>>>>,
    _phantom_data: PhantomData<C>,
}

impl<'a, C: Any> Iterator for Iter<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(next) => {
                    if let Some(value) = next.as_ref() {
                        return Some(unsafe { (&mut *(value.get())).downcast_ref::<C>().unwrap() });
                    }
                }
                None => return None,
            }
        }
    }
}

pub struct IterMut<'a, C> {
    inner: std::slice::Iter<'a, Option<UnsafeCell<Box<dyn Any>>>>,
    _phantom_data: PhantomData<C>,
}

impl<'a, C: Any> Iterator for IterMut<'a, C> {
    type Item = &'a mut C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(next) => {
                    if let Some(value) = next.as_ref() {
                        return Some(
                            (unsafe { &mut *(value.get()) })
                                .downcast_mut::<C>()
                                .unwrap(),
                        );
                    }
                }
                None => return None,
            }
        }
    }
}

impl From<&Entity> for Container {
    fn from(entity: &Entity) -> Self {
        Self {
            data: entity
                .archetype()
                .map(|&type_id| (type_id, Vec::with_capacity(1)))
                .collect::<HashMap<_, _>>(),
            removed: Vec::new(),
            len: 0,
        }
    }
}

/*
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
*/
