mod container;

use std::{
    any::TypeId,
    vec::Vec,
    marker::PhantomData
};

use container::Container;

use crate::{
    count,
    ecs::Component,
    recursive,
};

pub struct Context {
    pub name: *mut String,
}

impl Context {
    pub fn cast(&mut self) -> &mut String {
        unsafe {
            &mut *self.name
        }
    }
}

/// World implements a container for Systems, Entities and their Components and quering functionality
pub struct World {
    /// Entities container grouped by archetypes
    content: Vec<Container>,
    /// Spawn counter for Entity ID generation
    counter: u64,
    name: String,
}

// TODO: find a way how to get rid of it, maybe use some other constructor
unsafe impl Send for World {}
unsafe impl Sync for World {}

impl World {
    /// Create new empty World instance
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            counter: 0,
            name: String::from("MyWorld")
        }
    }

    /// Spawn single or multiple entities in the world
    pub fn spawn<T, I>(&mut self, iter: I) 
    where
        T: Archetype + Pattern,
        I: IntoIterator<Item = T>
    {
        let container = if let Some(container) = self.content
            .iter_mut()
            .find(|s| T::matches(s) && s.len() == T::len())
        {
           container
        } else {
            let container = Container::new::<T>();
            self.content.push(container);
            self.content.last_mut().unwrap()
        };

        for entity in iter {
            entity.store(container);
            self.counter += 1;
        }
    }

    /// Query stored components in the World
    pub fn query<'w, Q>(&'w self) -> impl Iterator<Item = <<Q as Query>::Iter as Iterator>::Item> + 'w
    where
        Q: Query<'w>,
    {
        let iter = self.content.iter()
            .filter(|&container| Q::matches(container))
            .map(|container| Q::select(&container))
            .flatten();

        Matches {
            iter,
        }
    }

    pub fn counter(&self) -> u64 {
        self.counter
    }

    /*
    pub fn get<T>(&mut self) -> &T
    where Self: Fetcher<T> {
        self.fetch()
    }
    */

    pub fn context(&mut self) -> Context {
        Context {
            name: (&mut self.name) as *mut String,
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/*
pub trait Fetcher<T> {
    fn fetch(&mut self) -> &T;
}

impl Fetcher<Context> for World {
    fn fetch(&mut self) -> &Context {
        &self.context
    }
}
*/

/*
pub trait Selector<'w> {
    type Iter: Iterator;
    type Component: Component;

    fn borrow(container: &'w Container) -> Self::Iter;
    fn matches(container: &'w Container) -> bool {
        container.has(TypeId::of::<Self::Component>())
    }
}

impl<'w, C> Selector<'w> for &'_ C
where
    C: Component,
{
    type Iter = std::slice::Iter<'w, C>;
    type Component = C;

    fn borrow(container: &'w Container) -> Self::Iter {
        container.get::<C>().unwrap().iter()
    }
}
*/

/// Trait definition of Entities with the same set of components
pub trait Archetype {
    fn store(self, container: &mut Container);
    fn map(container: &mut Container);
}

/// Trait definition of components set
pub trait Pattern {
    fn len() -> usize;
    fn matches(container: &Container) -> bool;
}

/// Trait definition of a Query
pub trait Query<'w> {
    type Iter: Iterator + 'w;

    fn select(container: &'w Container) -> Self::Iter;
    fn matches(container: &'w Container) -> bool;
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
    _phantom: PhantomData<&'w ()>,
}

/// Trait defenition of Selector to control mutability of borrows
pub trait Selector<'w> {
    type Iter: Iterator;
    type Component: Component;

    fn borrow(container: &'w Container) -> Self::Iter;
    fn matches(container: &'w Container) -> bool {
        container.has(TypeId::of::<Self::Component>())
    }
}

impl<'w, C> Selector<'w> for &'_ C
where
    C: Component,
{
    type Iter = std::slice::Iter<'w, C>;
    type Component = C;

    fn borrow(container: &'w Container) -> Self::Iter {
        container.get::<C>().unwrap().iter()
    }
}

impl<'w, C> Selector<'w> for &'_ mut C
where
    C: Component,
{
    type Iter = std::slice::IterMut<'w, C>;
    type Component = C;

    fn borrow(container: &'w Container) -> Self::Iter {
        container.get_mut::<C>().unwrap().iter_mut()
    }
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
            fn store(self, container: &mut Container) {
                let ($($i,)*) = self;
                $(
                    container.push::<$i>($i);
                )*
            }
            fn map(container: &mut Container) {
                $(
                    container.init::<$i>();
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

            fn matches(container: &Container) -> bool {
                $(container.has(TypeId::of::<$i>()))&&*
            }
        }

        impl<'w, $($i),*> Query<'w> for ($($i,)*)
        where
            $($i: Selector<'w> + 'w,)*
        {
            type Iter = Zipper<'w, ($($i::Iter,)*)>;
            // type Iter = Zipper<'w, ($(std::slice::Iter<'w, $i>,)*)>;

            fn select(container: &'w Container) -> Self::Iter {
                Zipper {
                    tuple: ($({$i::borrow(container)},)*),
                    // tuple: ($(container.get::<$i::Component>().unwrap().into_iter(),)*),
                    _phantom: PhantomData,
                }
            }

            fn matches(container: &'w Container) -> bool
            {
                $(
                    $i::matches(container)
                )&&*
            }
        }

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
                        Some(item) => item,
                    };

                )*
                Some(($($i,)*))
            }
        }
    }
}

recursive!(impl_tuples, A, B, C, D, E, F); //, E, F, G, H, I, J, K, L, M, N, O, P);

#[cfg(test)]
mod tests {
    use super::World;

    struct Armor(u32);
    struct Health(u32);
    struct Speed(u32);
    struct Damage(u32);
    struct Weight(u32);

    fn spawn() -> World {
        let mut world = World::new();
        world.spawn(Some((Armor(100), Health(100), Damage(300))));
        world.spawn(Some((Health(80), Speed(10))));
        world.spawn(Some((Speed(50), Damage(45))));
        world.spawn(Some((Damage(600), Armor(10))));

        let bulk = (0..9).map(|_| (Speed(35), Weight(5000)));
        world.spawn(bulk);

        world
    }

    #[test]
    fn spawn_and_query() {
        let world = spawn();

        let mut iter = world.query::<(&Armor, &Damage)>();

        let item = iter.next();
        assert_eq!(item.is_some(), true);

        if let Some ((armor, damage)) = item {
            assert_eq!(armor.0, 100); // Armor(100)
            assert_eq!(damage.0, 300); // Damage(300)
        }

        let item = iter.next();
        assert_eq!(item.is_some(), true);

        let item = item.unwrap();
        assert_eq!(item.0.0, 10); // Armor(10)
        assert_eq!(item.1.0, 600); // Damage(600)

        let item = iter.next();
        assert_eq!(item.is_some(), false);
    }

    #[test]
    fn spawn_and_modify() {
        let world = spawn();
        {
            let iter = world.query::<(&mut Speed,)>();
            for (speed,) in iter {
                speed.0 = 123;
            }
        }
        {
            let iter = world.query::<(&Speed,)>();
            for (speed,) in iter {
                println!("speed is {}", speed.0);
                assert_eq!(speed.0, 123);
            }
        }
    }
}
