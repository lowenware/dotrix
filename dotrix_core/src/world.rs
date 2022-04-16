mod container;

use std::{any::TypeId, collections::HashMap, marker::PhantomData, vec::Vec};

use container::Container;

use crate::{
    count,
    ecs::{Component, Entity},
    recursive,
};

/// Service to store and manage entities
pub struct World {
    /// Entities container grouped by archetypes
    content: Vec<Container>,
    /// Index of the container that holds Entity
    index: HashMap<Entity, usize>,
    /// Spawn counter for Entity ID generation
    counter: u64,
    /// Buffer to track spawned entities
    spawned: Vec<Entity>,
}

impl World {
    /// Create new empty World instance
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            index: HashMap::new(),
            counter: 0,
            spawned: Vec::with_capacity(16),
        }
    }

    /// Spawn single or multiple entities in the world
    ///
    /// Entity appears only when you spawn it in the [`World`] as a tuple of components.
    ///
    /// ```no_run
    /// use dotrix_core::{
    ///     ecs::Mut,
    ///     World,
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
    pub fn spawn<T, I>(&mut self, iter: I) -> SpawnedIter
    where
        T: Archetype + Pattern,
        I: IntoIterator<Item = T>,
    {
        let (index, container) = if let Some((index, container)) = self
            .content
            .iter_mut()
            .enumerate()
            .find(|(_, s)| T::matches(s) && s.len() == T::len() + 1)
        // + Entity
        {
            (index, container)
        } else {
            let container = Container::new::<T>();
            let index = self.content.len();
            self.content.push(container);
            (index, self.content.last_mut().unwrap())
        };

        self.spawned.clear();

        for tuple in iter {
            let entity_id = self.counter;
            let entity = Entity::from(entity_id);
            tuple.store(container, entity_id);
            self.index.insert(entity, index);
            self.counter += 1;
            self.spawned.push(entity)
        }

        let count = self.spawned.len();
        let first = self.spawned[0];
        let last = self.spawned[count - 1];

        SpawnedIter {
            iter: self.spawned.iter(),
            first,
            last,
            count,
        }
    }

    /// Query entities from the World
    ///
    /// ## Example:
    /// ```no_run
    /// use dotrix_core::{
    ///     ecs::Mut,
    ///     World,
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
    ///         cmp2.0 = 0;
    ///     }
    /// }
    /// ```
    pub fn query<'w, Q>(
        &'w self,
    ) -> impl Iterator<Item = <<Q as Query>::Iter as Iterator>::Item> + 'w
    where
        Q: Query<'w>,
    {
        let iter = self
            .content
            .iter()
            .filter(|&container| Q::matches(container))
            .flat_map(|container| Q::select(container));

        Matches { iter }
    }

    /// Get componets dor specified entity
    ///
    /// ## Example:
    /// ```no_run
    /// use dotrix_core::ecs::{Entity, Mut};
    /// use dotrix_core::World;
    ///
    /// // First component
    /// struct Component1(u32);
    ///
    /// // Second component
    /// struct Component2(u32);
    ///
    /// fn my_system(mut world: Mut<World>) {
    ///     let entity = Entity::from(0); // mockup entity
    ///     if let Some((c1, c2)) = world.get::<(&Component1, &mut Component2)>(entity) {
    ///         // do stuff
    ///         println!("Component1({}), Component2({})", c1.0, c2.0);
    ///     }
    /// }
    pub fn get<'w, Q>(&'w self, entity: Entity) -> Option<Q>
    where
        Q: Query<'w>,
    {
        self.index.get(&entity).map(|&index| {
            let entity_offset = self.content[index]
                .get::<Entity>()
                .unwrap()
                .iter()
                .position(|&e| e == entity)
                .unwrap();
            Q::pick(&self.content[index], entity_offset)
        })
    }

    /// Exiles an entity from the world
    ///
    /// ## Example:
    /// ```no_run
    /// use dotrix_core::{
    ///     ecs::{Mut, Entity},
    ///     World,
    /// };
    /// // First component
    /// struct Component(u32);
    ///
    /// fn exile_system(mut world: Mut<World>) {
    ///     let query = world.query::<(&Entity, &Component)>();
    ///     let mut to_exile = Vec::new();
    ///     for (entity, cmp) in query {
    ///         if cmp.0 < 10 {
    ///             to_exile.push(*entity);
    ///         }
    ///     }
    ///     for entity in to_exile.into_iter() {
    ///         world.exile(entity);
    ///     }
    /// }
    /// ```
    ///
    /// Never exile entities inside of the query loop. Store them somewhere instead and exile
    /// afterwards
    pub fn exile(&mut self, entity: Entity) {
        let index = if let Some(index) = self.index.get(&entity) {
            *index
        } else {
            return;
        };

        if index >= self.content.len() {
            return;
        }

        let entity_index = if let Some(entities) = self.content[index].get::<Entity>() {
            if let Some((index, _)) = entities
                .iter()
                .enumerate()
                .find(|(_index, e)| **e == entity)
            {
                index
            } else {
                return;
            }
        } else {
            return;
        };
        self.content[index].remove(entity_index);
        self.index.remove(&entity);
    }

    /// Returns current value of entities counter
    pub fn counter(&self) -> u64 {
        self.counter
    }

    /// Clear all entities from the world
    pub fn clear(&mut self) {
        self.content.clear();
        self.spawned.clear();
        self.index.clear();
    }

    /// Clear entities from the world and reset to initial state
    pub fn reset(&mut self) {
        self.clear();
        self.counter = 0;
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
    fn store(self, container: &mut Container, entity_id: u64);
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
    /// Pick specific entity by its index in container
    fn pick(container: &'w Container, entity_index: usize) -> Self;
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
    fn borrow_by_index(container: &'w Container, entity_index: usize) -> Self;
    fn matches(container: &'w Container) -> bool {
        container.has(TypeId::of::<Self::Component>())
    }
}

impl<'w, C> Selector<'w> for &'w C
where
    C: Component,
{
    type Iter = std::slice::Iter<'w, C>;
    type Component = C;

    fn borrow(container: &'w Container) -> Self::Iter {
        container.get::<C>().unwrap().iter()
    }

    fn borrow_by_index(container: &'w Container, entity_index: usize) -> Self {
        &container.get::<C>().unwrap()[entity_index]
    }
}

impl<'w, C> Selector<'w> for &'w mut C
where
    C: Component,
{
    type Iter = std::slice::IterMut<'w, C>;
    type Component = C;

    fn borrow(container: &'w Container) -> Self::Iter {
        container.get_mut::<C>().unwrap().iter_mut()
    }

    fn borrow_by_index(container: &'w Container, entity_index: usize) -> Self {
        &mut container.get_mut::<C>().unwrap()[entity_index]
    }
}

/// Iterator over Vertices Attributes
pub struct SpawnedIter<'a> {
    iter: std::slice::Iter<'a, Entity>,
    first: Entity,
    last: Entity,
    count: usize,
}

impl<'a> SpawnedIter<'a> {
    /// get first spawned entity
    pub fn first(&mut self) -> Entity {
        self.first
    }

    /// get number of spawned entities
    pub fn entities(&self) -> usize {
        self.count
    }
}

impl<'a> Iterator for SpawnedIter<'a> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
    fn last(self) -> Option<Entity> {
        Some(self.last)
    }
    fn count(self) -> usize {
        self.entities()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.count;
        (size, Some(size))
    }
}

impl<'a> ExactSizeIterator for SpawnedIter<'a> {
    fn len(&self) -> usize {
        self.count
    }
}

impl<'a> From<SpawnedIter<'a>> for Vec<Entity> {
    fn from(iter: SpawnedIter<'a>) -> Self {
        let mut result = Vec::with_capacity(iter.entities());
        for e in iter {
            result.push(e);
        }
        result
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
            fn store(self, container: &mut Container, entity_id: u64) {
                let ($($i,)*) = self;

                container.push::<Entity>(Entity::from(entity_id));
                $(
                    container.push::<$i>($i);
                )*
            }
            fn map(container: &mut Container) {
                container.init::<Entity>();
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
            fn pick(container: &'w Container, entity_index: usize) -> ($($i,)*) {
                ($({$i::borrow_by_index(container, entity_index)},)*)
            }

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

recursive!(impl_tuples, A, B, C, D, E, F, G, H); //, E, F, G, H, I, J, K, L, M, N, O, P);

#[cfg(test)]
mod tests {
    use super::World;
    use crate::ecs::Entity;

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    struct Armor(u32);
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
        assert!(item.is_some());

        if let Some((armor, damage)) = item {
            assert_eq!(armor.0, 100); // Armor(100)
            assert_eq!(damage.0, 300); // Damage(300)
        }

        let item = iter.next();
        assert!(item.is_some());

        let item = item.unwrap();
        assert_eq!(item.0 .0, 10); // Armor(10)
        assert_eq!(item.1 .0, 600); // Damage(600)

        let item = iter.next();
        assert!(item.is_none());
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
                assert_eq!(speed.0, 123);
            }
        }
    }

    #[test]
    fn spawn_and_exile() {
        let mut world = spawn();
        {
            let iter = world.query::<(&Entity, &mut Armor)>();
            let mut entity_to_delete = None;
            let mut entities_before = 0;
            for (entity, armor) in iter {
                if armor.0 == 100 {
                    entity_to_delete = Some(*entity);
                }
                entities_before += 1;
            }
            assert!(entity_to_delete.is_some());

            world.exile(entity_to_delete.unwrap());

            let iter = world.query::<(&Entity, &mut Armor)>();
            let mut entities_after = 0;
            for (entity, _armor) in iter {
                assert_ne!(*entity, entity_to_delete.unwrap());
                entities_after += 1;
            }

            assert_eq!(entities_before - 1, entities_after);
        }
    }

    #[test]
    fn spawn_and_get_by_entity() {
        let world = spawn();
        let entity = Entity::from(0);
        let query = world.get::<(&Armor, &Health)>(entity);
        assert_eq!(query.is_some(), true);
        if let Some((&armor, &health)) = query {
            assert_eq!(armor, Armor(100));
            assert_eq!(health, Health(100));
        }
    }

    #[test]
    fn spawn_and_check_entities() {
        let mut world = World::default();
        let mut spawned = world.spawn(Some((Armor(1),)));

        assert_eq!(spawned.next(), Some(Entity::from(0)));
        assert_eq!(spawned.first(), Entity::from(0));
        assert_eq!(spawned.last(), Some(Entity::from(0)));

        let spawned: Vec<Entity> = world
            .spawn([
                (Armor(2), Health(100)),
                (Armor(4), Health(80)),
                (Armor(4), Health(90)),
            ])
            .into();

        assert_eq!(spawned.len(), 3);

        for i in 0..3 {
            assert_eq!(spawned[i], Entity::from(i as u64 + 1));
        }
    }
}
