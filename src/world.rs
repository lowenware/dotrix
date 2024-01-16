mod storage;

use std::sync::{Arc, Condvar, Mutex};
use std::{any::TypeId, collections::HashMap, marker::PhantomData};

use crate::recursive;
use crate::utils::{Id, Lock, TypeLock};
pub use storage::{Entity, IntoEntity};

#[derive(Default, Debug, Eq, PartialEq)]
struct Index {
    /// storage::Container Index
    container: usize,
    /// Entity Index
    address: usize,
}

/// Service to store and manage entities
pub struct World {
    /// Entities container grouped by archetypes
    content: Vec<storage::Container>,
    /// Index of the container that holds Entity
    index: HashMap<Id<Entity>, Index>,
    // /// Spawn counter for Entity ID generation
    // next_id: u64,
    /// Lock for multithread safety
    lock: Arc<(Mutex<TypeLock>, Condvar)>,
}

impl World {
    /// Create new empty World instance
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            index: HashMap::new(),
            // next_id: 1,
            lock: Arc::new((Mutex::new(TypeLock::new()), Condvar::new())),
        }
    }

    /// Spawn single or multiple entities in the world
    ///
    /// Returns number of spawned entities
    pub fn spawn_and_count<T, I>(&mut self, entries: I) -> usize
    where
        T: IntoEntity,
        I: IntoIterator<Item = T>,
    {
        self.spawn(entries).count()
    }

    /// Returns an iterator over entity IDs
    ///
    /// Actual spawning of the entity occurs on iterator consuming
    pub fn spawn<T, I>(&mut self, entries: I) -> SpawnIter<I, T>
    where
        T: IntoEntity,
        I: IntoIterator<Item = T>,
    {
        SpawnIter {
            entries: entries.into_iter(),
            world: self,
            container_index: None,
        }
    }

    fn lock<'w, Q: Query<'w>>(&self) {
        let locks = Q::locks();
        let (mutex, cvar) = &*self.lock;
        let mut lock_manager = mutex.lock().unwrap();
        while !lock_manager.lock(&locks) {
            lock_manager = cvar.wait(lock_manager).unwrap();
        }
    }

    /// Returns iterator over entities defined by Query pattern
    pub fn query<'w, Q>(
        &'w self,
    ) -> impl Iterator<Item = <<Q as Query>::Iter as Iterator>::Item> + 'w
    where
        Q: Query<'w>,
    {
        self.lock::<Q>();

        let iter = self
            .content
            .iter()
            .filter(|container| Q::matches(container))
            .flat_map(|container| Q::select(container));

        QueryIter {
            iter,
            locks: Q::locks(),
            lock_manager: Arc::clone(&self.lock),
        }
    }

    /// Execute a system for each entity in the world
    pub fn execute<'w, Q, S>(&'w self, system: S)
    where
        Q: Query<'w> + 'w,
        S: Fn(<<Q as Query<'_>>::Iter as Iterator>::Item),
    {
        // TODO: parallelize
        for item in self.query::<Q>() {
            system(item);
        }
    }

    /// Get componets dor specified entity
    pub fn get<'w, Q>(&'w self, id: &Id<Entity>) -> Option<Q>
    where
        Q: Query<'w>,
    {
        self.lock::<Q>();
        self.index
            .get(id)
            .map(|index| Q::pick(&self.content[index.container], index.address))
    }

    /// Exiles an entity from the world
    pub fn exile(&mut self, id: &Id<Entity>) -> Option<Entity> {
        self.index
            .get(id)
            .map(|index| self.content[index.container].remove(index.address))
    }

    /// Clear all entities from the world
    pub fn clear(&mut self) {
        self.content.clear();
        self.index.clear();
    }

    /// Clear entities from the world and reset to initial state
    pub fn reset(&mut self) {
        self.clear();
        // self.next_id = 0;
    }

    fn find_container_for_entity(&self, entity: &Entity) -> Option<usize> {
        self.content
            .iter()
            .enumerate()
            .find(|(_, container)| container.matches(&mut entity.archetype()))
            .map(|(index, _)| index)
    }

    fn create_container_for_entity(&mut self, entity: &Entity) -> usize {
        let index = self.content.len();
        self.content.push(storage::Container::from(entity));
        index
    }

    // fn next_id(&mut self) -> u64 {
    //    let next_id = self.next_id;
    //    self.next_id += 1;
    //    next_id
    // }
}

/// Abstraction for queries inoked by [`World::query`]
pub trait Query<'w> {
    type Iter: Iterator + 'w;
    /// Returns vector of locks necesary for the query execution
    fn locks() -> Vec<Lock>;
    /// Selects entities from container
    fn select(container: &'w storage::Container) -> Self::Iter;
    /// Checks if [`Query`] matches the [`storage::Container`]
    fn matches(container: &'w storage::Container) -> bool;
    /// Pick specific entity by its index in container
    fn pick(container: &'w storage::Container, entity_index: usize) -> Self;
}

/// Trait defenition of Selector to control mutability of borrows
pub trait Selector<'w> {
    type Iter: Iterator + 'w;
    type Component: std::any::Any;

    fn borrow(container: &'w storage::Container) -> Self::Iter;
    fn borrow_one(container: &'w storage::Container, entity_index: usize) -> Self;
    fn lock() -> Lock;
}

impl<'w, C> Selector<'w> for &'w C
where
    C: Send + Sync + 'static,
{
    // type Iter = std::slice::Iter<'w, C>;
    type Iter = storage::Iter<'w, C>;
    type Component = C;

    fn borrow(container: &'w storage::Container) -> Self::Iter {
        container.iter::<C>()
    }

    fn borrow_one(container: &'w storage::Container, entity_index: usize) -> Self {
        container.get::<C>(entity_index).unwrap()
    }

    fn lock() -> Lock {
        Lock::ReadOnly(TypeId::of::<C>())
    }
}

impl<'w, C> Selector<'w> for &'w mut C
where
    C: Send + Sync + 'static,
{
    type Iter = storage::IterMut<'w, C>;
    type Component = C;

    fn borrow(container: &'w storage::Container) -> Self::Iter {
        unsafe { container.iter_mut::<C>() }
    }

    fn borrow_one(container: &'w storage::Container, entity_index: usize) -> Self {
        unsafe { container.get_mut::<C>(entity_index).unwrap() }
    }

    fn lock() -> Lock {
        Lock::ReadWrite(TypeId::of::<C>())
    }
}

/// Iterator converting a tuple of iterators into the Iterator of a tuple
pub struct Zipper<'w, T> {
    tuple: T,
    _lifetime: PhantomData<&'w ()>,
}

macro_rules! impl_queries {
    ($($i: ident),*) => {
        impl<'w, $($i),*> Query<'w> for ($($i,)*)
        where
            $($i: Selector<'w> + 'w,)*
        {
            type Iter = Zipper<'w, ($($i::Iter,)*)>;

            fn pick(container: &'w storage::Container, entity_index: usize) -> ($($i,)*) {
                ($({$i::borrow_one(container, entity_index)},)*)
            }

            fn select(container: &'w storage::Container) -> Self::Iter {
                Zipper {
                    tuple: ($({$i::borrow(container)},)*),
                    // tuple: ($(container.get::<$i::Component>().unwrap().into_iter(),)*),
                    _lifetime: PhantomData,
                }
            }

            fn matches(container: &'w storage::Container) -> bool
            {
                $(
                    container.contains(TypeId::of::<$i::Component>())
                )&&*
            }

            fn locks() -> Vec<Lock> {
                vec![
                    $(
                        $i::lock(),
                    )*
                ]
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
recursive!(impl_queries, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

/// Iterator or Query result
pub struct QueryIter<I>
where
    I: Iterator,
{
    iter: I,
    locks: Vec<Lock>,
    lock_manager: Arc<(Mutex<TypeLock>, Condvar)>,
}

impl<I> Drop for QueryIter<I>
where
    I: Iterator,
{
    fn drop(&mut self) {
        let (mutex, cvar) = &*self.lock_manager;

        let mut lock_manager = mutex.lock().expect("Mutex failed to lock");

        lock_manager.unlock(self.locks.as_slice());
        cvar.notify_all();
    }
}

impl<I> Iterator for QueryIter<I>
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

pub struct SpawnIter<'a, I, T>
where
    T: IntoEntity,
    I: IntoIterator<Item = T>,
{
    entries: I::IntoIter,
    world: &'a mut World,
    container_index: Option<usize>,
}

impl<'a, I, T> Iterator for SpawnIter<'a, I, T>
where
    T: IntoEntity,
    I: IntoIterator<Item = T>,
{
    type Item = Id<Entity>;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next().map(|entry| {
            let id = Id::<Entity>::new();
            let volatile = T::volatile();
            let entity = entry.entity().with(id);
            self.container_index = self
                .container_index
                // container was previously set
                .and_then(|i| {
                    if volatile && !self.world.content[i].matches(&mut entity.archetype()) {
                        // we may need different container for the entity
                        return None;
                    }
                    Some(i)
                })
                .or_else(|| self.world.find_container_for_entity(&entity))
                .or_else(|| Some(self.world.create_container_for_entity(&entity)));

            let index = {
                let container_index = self.container_index.unwrap();
                let container = &mut self.world.content[container_index];
                let address = container.store(entity);
                Index {
                    container: container_index,
                    address,
                }
            };
            self.world.index.insert(id, index);
            id
        })
    }
    fn count(self) -> usize {
        let mut result = 0;
        for _ in self {
            result += 1;
        }
        result
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}

#[cfg(test)]
mod tests {
    use super::World;

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
    fn can_spawn_and_query() {
        let mut world = spawn();
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
}

/*
#[cfg(test)]
mod tests {
    use super::World;
    use crate::{World, Entity;

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
            let iter = world.query::<(&mut Speed,)>()D;
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
        assert!(query.is_some());
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

        for (i, ent) in spawned.iter().enumerate() {
            assert_eq!(ent, &Entity::from(i as u64 + 1));
        }
    }
}
*/
