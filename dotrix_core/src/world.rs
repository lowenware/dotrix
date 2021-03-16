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


/// Service to store and manage entities
///
/// To enable entities management in your game, add the service service using [`crate::Dotrix`]
/// builder, so it can be accessed in [`crate::systems`].
///
/// ```no_run
/// use dotrix_core::{
///     Dotrix,
///     services::World
/// };
///
/// // in fn main()
/// Dotrix::application("My Game")
///     .with_service(World::new())
///     .run()
/// ```
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
    ///
    /// Entity appears only when you spawn it in the [`World`] as a tuple of [`crate::components`].
    ///
    /// ```no_run
    /// use dotrix_core::{
    ///     ecs::Mut,
    ///     services::World,
    /// };
    ///
    /// // First component
    /// struct Component1(u32);
    /// // Second component
    /// struct Component2(u32);
    ///
    /// fn spawn(mut world: Mut<World>) {
    ///     // Spawn single entity in the world
    ///     world.spawn(
    ///         Some( // Iterator
    ///             (Component1, Component2) // Your Entity
    ///         )
    ///     );
    ///
    ///     // Bulk spawning of entities of the same archetype
    ///     world.spawn((0..123).map(|i| (Component1(i),)));
    /// }
    /// ```
    ///
    /// _NOTE: if your entity has only one component, don't forget to put trailing comma after it
    /// as it is shown in the example above. Otherwise it won't be parsed by Rust compiler as
    /// a tuple.
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

    /// Query entities from the World
    ///
    /// ## Example:
    /// ```no_run
    /// use dotrix_core::{
    ///     ecs::Mut,
    ///     services::World,
    /// };
    /// // First component
    /// struct Component1(u32);
    ///
    /// // Second component
    /// struct Component2(u32);
    ///
    /// fn my_system(mut world: Mut<World>) {
    ///     let query = world.query::<(&Component1, &mut Component2)>();
    ///     for (cmp1, cmp2) in query {
    ///         println!("Reset value of Component 2 for Component 1 of {}", cmp1.0);
    ///         cmp2.0 = 0;
    ///     }
    /// }
    /// ```
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

    /// Returns current value of entities counter
    pub fn counter(&self) -> u64 {
        self.counter
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Abstraction for Entities with the same set of components
pub trait Archetype {
    /// Stores archetype in a [`Container`]
    fn store(self, container: &mut Container);
    /// Prepares the [`Container`] to store the archetype
    fn map(container: &mut Container);
}

/// Abstraction for a set of components
pub trait Pattern {
    /// Returns number of components in the [`Pattern`]
    fn len() -> usize;
    /// Checks if [`Pattern`] matches the [`Archetype`]
    fn matches(container: &Container) -> bool;
}

/// Abstraction for queries inoked by [`World::query`]
pub trait Query<'w> {
    type Iter: Iterator + 'w;

    /// Selects entities from container
    fn select(container: &'w Container) -> Self::Iter;
    /// Checks if [`Query`] matches the [`Container`]
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
