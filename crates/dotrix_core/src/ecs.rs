use std::{
    any::{Any, TypeId},
    collections::HashMap,
    vec::Vec,
};


/// Entity structure has only id field and represent an agregation of components
pub struct Entity(u64);

/// Any data structure can be a component
pub trait Component: Send + Sync + 'static {
    fn name(&self) -> &'static str;
}
impl<T: Send + Sync + 'static> Component for T {
    fn name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

type ComponentId = TypeId;

/// Trait for ECS systems
pub trait System {
    fn startup(&mut self, world: &mut World);

    fn run_cycle(&mut self, world: &mut World);
}

/// World structure implements root level container for entities and components and query
/// functions
pub struct World {
    /// Entities container grouped by archetypes
    containers: Vec<Container>,
    /// Entities index for quicker access
    index: Vec<EntityIndex>,
    /// Spawn counter for Entity ID generation
    counter: u64,
}

/// Archetype is a structure-of-arrays (SOA) container of entities and their components, grouping
/// Entities with same set of components
struct Container {
    entities: Vec<Entity>,
    components: HashMap<ComponentId, Vec<Box<dyn Component>>>,
}

/// EntityIndex contains offsets of entities in world.archetypes and archetype.entities
struct EntityIndex {
    entity: Entity,
    archetype_offset: usize,
    entity_offset: usize,
}

impl World {
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
            index: Vec::new(),
            counter: 1,
        }
    }

    pub fn spawn<T, I>(&mut self, iter: I) 
    where
        T: Archetype,
        I: IntoIterator<Item = T>
    {
        let mut container = if
            let Some(container) = self.containers
                .iter_mut()
                .find(|c| c.is_for::<T>())
        {
            container
        } else {
            let container = Container::new::<T>();
            self.containers.push(container);
            self.containers.last_mut().unwrap()
        };

        for components in iter {
            container.push(Entity(self.counter), components);
            self.counter += 1;
        }
    }

    //pub fn query(&mut self, ) -> ??? {

    //}

    // pub fn first(&mut self, ) -> ??? {

    // }

    // pub fn exile(&mut self, ) -> Result<Ok, Err> {

    // }
}

impl Container {
    pub fn new<T: Archetype>() -> Self {
        let mut result = Self {
            entities: Vec::new(),
            components: HashMap::new(),
        };
        T::map(&mut result.components);
        result
    }

    pub fn is_for<T: Archetype>(&self) -> bool {
        T::is_for(&self.components)
    }

    pub fn push<T: Archetype>(&mut self, entity: Entity, components: T) {
        self.entities.push(entity);
        components.store(&mut self.components);
    }
}

pub trait Archetype {
    fn store(self, hashmap: &mut HashMap<ComponentId, Vec<Box<dyn Component>>>);
    fn map(hashmap: &mut HashMap<ComponentId, Vec<Box<dyn Component>>>);
    fn is_for(hashmap: &HashMap<ComponentId, Vec<Box<dyn Component>>>) -> bool;
}

macro_rules! count {
    () => (0usize);
    ( $x:tt, $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! impl_tuple_archetype {
    ($($comp: ident),*) => {
        impl<$($comp),*> Archetype for ($($comp,)*)
        where
            $(
                $comp: Component,
            )*
        {

            #[allow(non_snake_case)]
            fn store(self, hashmap: &mut HashMap<ComponentId, Vec<Box<dyn Component>>>) {
                let ($($comp,)*) = self;
                $(
                    if let Some(mut vector) = hashmap.get_mut(&TypeId::of::<$comp>()) {
                        vector.push(Box::new($comp));
                    } else {
                        panic!("ECS storage is corrupted");
                    }
                )*
            }

            fn map(hashmap: &mut HashMap<ComponentId, Vec<Box<dyn Component>>>) {
                $(
                    hashmap.insert(TypeId::of::<$comp>(), Vec::new());
                )*
            }

            fn is_for(hashmap: &HashMap<ComponentId, Vec<Box<dyn Component>>>) -> bool {
                hashmap.len() == count!($($comp,)*)
                $(
                    && hashmap.get(&TypeId::of::<$comp>()).is_some()
                )*
            }
        }
    }
}


impl Container {
    /*
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: HashMap::new(),
        }
    }
    */
}

impl EntityIndex {
    pub fn new(entity: Entity, archetype_offset: usize, entity_offset: usize) -> Self {
        Self {
            entity,
            archetype_offset,
            entity_offset,
        }
    }
}

#[cfg(test)]
mod tests {

    struct Health {}
    struct Armor {}

    #[test]
    fn it_works() {
        /*
        let mut world = World::new();
        let archtype = (0..99).map(|_| (Health {}, Armor{}));

        world.spawn(archtype);

        world.spwan(Some((Health {}, Armor{})));
        */
        assert_eq!(2 + 2, 4);

    }
}

/// Recursive macro treating arguments as a progression
///
/// Expansion of recursive!(macro, A, B, C) is equivalent to the expansion of sequence
/// macro!()
/// macro!(A)
/// macro!(A, B)
/// macro!(A, B, C)
#[macro_export]
macro_rules! recursive {
    ($macro: ident, $args: ident) => {
        $macro!{$args}
        $macro!{}
    };
    ($macro: ident, $first: ident, $($rest: ident),*) => {
        $macro!{$first, $($rest),*}
        recursive!{$macro, $($rest),*}
    };
}

// Implement traits for 16 archetypes
recursive!(impl_tuple_archetype, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
