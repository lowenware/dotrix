mod archetype;
mod world;

use std::any::{Any, TypeId};
use std::collections::{hash_map, HashMap};

use dotrix_types::id::{Id, NameSpace};

pub use world::World;

pub const NAMESPACE: u64 = 0x4040;

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

impl NameSpace for Entity {
    fn namespace() -> u64 {
        NAMESPACE
    }
}

pub struct Archetype<'a> {
    inner: hash_map::Keys<'a, TypeId, Box<dyn Any>>,
    len: usize,
}

impl<'a> Archetype<'a> {
    fn len(&self) -> usize {
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
