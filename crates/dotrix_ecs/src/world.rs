use std::{
    any::TypeId,
    vec::Vec,
};

use crate::{
    container::Container,
    count,
    ecs::Component,
    recursive,
};

/// World implements a storage for Systems, Entities and their Components and quering functionality
pub struct World {
    /// Entities container grouped by archetypes
    content: Vec<Container>,
    /// Spawn counter for Entity ID generation
    counter: u64,
}

impl World {
    /// Create new empty World instance
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            counter: 0,
        }
    }

    /// Spawn single or multiple entities in the world
    pub fn spawn<T, I>(&mut self, iter: I) 
    where
        T: Archetype + Pattern,
        I: IntoIterator<Item = T>
    {
        let storage = if let Some(storage) = self.content
            .iter_mut()
            .find(|s| T::matches(s) && s.len() == T::len())
        {
           storage
        } else {
            let storage = Container::new::<T>();
            self.content.push(storage);
            self.content.last_mut().unwrap()
        };

        for entity in iter {
            entity.store(storage);
            self.counter += 1;
        }
    }

    /// Query stored components in the World
    pub fn query<'w, Q>(&'w self) -> impl Iterator<Item = <<Q as Query>::Iter as Iterator>::Item> + 'w
    where
        Q: Query<'w> + Pattern,
    {
        let iter = self.content.iter()
            .filter(|&storage| Q::matches(storage))
            .map(|storage| Q::select(&storage))
            .flatten();

        Matches {
            iter,
        }
    }
}

/// Trait definition of Entities with the same set of components
pub trait Archetype {
    fn store(self, storage: &mut Container);
    fn map(storage: &mut Container);
}

/// Trait definition of components set
pub trait Pattern {
    fn len() -> usize;
    fn matches(storage: &Container) -> bool;
}

/// Trait definition of a Query
pub trait Query<'w> {
    type Iter: Iterator + 'w;

    fn select(storage: &'w Container) -> Self::Iter;
}

/// Iterator or Query result
pub struct Matches<I> {
    iter: I,
}

impl<'w, I> Iterator for Matches<I>
where
    I: Iterator,
{
    type Item = I::Item;
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// Iterator converting a tuple of iterators into the Iterator of a tuple
pub struct Zipper<'w, T> {
    tuple: T,
    _m: std::marker::PhantomData<&'w ()>,
}

/// Macros implementing all necessary archetyoes, patterns, querries and iterators for different
/// types of tuples
#[macro_export]
macro_rules! impl_tuples {
    ($($i: ident),*) => {
        impl<$($i),*> Archetype for ($($i,)*)
        where
            $(
                $i: Component,
            )*
        {
            #[allow(non_snake_case)]
            fn store(self, storage: &mut Container) {
                let ($($i,)*) = self;
                $(
                    storage.push::<$i>($i);
                )*
            }
            fn map(storage: &mut Container) {
                $(
                    storage.init::<$i>();
                )*
            }
        }

        impl<$($i),*> Pattern for ($($i,)*)
        where
            $($i: Component,)*
        {
            fn len() -> usize {
                count!($($i,)*)
            }

            fn matches(storage: &Container) -> bool {
                $(storage.has(TypeId::of::<$i>()))&&*
            }
        }

        impl<'w, $($i),*> Query<'w> for ($($i,)*)
        where
            $($i: Component + 'w,)*
        {
            type Iter = Zipper<'w, ($(std::slice::Iter<'w, $i>,)*)>;

            fn select(storage: &'w Container) -> Self::Iter {
                Zipper {
                    tuple: ($(storage.get::<$i>().unwrap().into_iter(),)*),
                    _m: std::marker::PhantomData,
                }
            }
        }

        /// $i is Iterator
        #[allow(non_snake_case)]
        impl<'w, $($i),*> Iterator for Zipper<'w, ($($i,)*)>
        where
            $($i: Iterator + 'w,)*
        {
            type Item = ($($i::Item,)*);

            fn next(&mut self) -> Option<Self::Item> {
                let ($(ref mut $i,)*) = self.tuple;
                $(
                    let $i = match $i.next() {
                        None => return None,
                        Some(item) => item
                    };
                )*
                Some(($($i,)*))
            }
        }
    }
}

recursive!(impl_tuples, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

