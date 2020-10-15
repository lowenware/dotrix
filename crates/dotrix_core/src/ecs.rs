use std::{
    collections::HashMap,
    vec::Vec,
};

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

type ComponentId = &'static str;

/// Trait for ECS systems
pub trait System {
    fn startup(&mut self, world: &mut World);

    fn run_cycle(&mut self, world: &mut World);
}

/// World structure implements root level container for entities and components and query
/// functions
pub struct World {
    /// Entities container grouped by archetypes
    archetypes: Vec<Archetype>,
    /// Entities index for quicker access
    index: Vec<EntityIndex>,
    /// Spawn counter for Entity ID generation
    counter: u64,
}

/// Archetype is a structure-of-arrays (SOA) container of entities and their components, grouping
/// Entities with same set of components
struct Archetype {
    entities: Vec<Entity>,
    components: HashMap<ComponentId, Vec<Box<Component>>>,
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
            archetypes: Vec::new(),
            index: Vec::new(),
            counter: 1,
        }
    }

    pub fn spawn<T, I>(&mut self, i: I) 
    where
        T: Component + ListComponents,
        I: IntoIterator<Item = T>
    {
        for e in i {
            println!("Entity: {}, {}:", self.counter, std::any::type_name::<T>());
            self.counter += 1;
            for c in e.list() {
                println!("    {}", c.as_ref().name());
            }
        }
    }

    //pub fn query(&mut self, ) -> ??? {

    //}

    // pub fn first(&mut self, ) -> ??? {

    // }

    // pub fn exile(&mut self, ) -> Result<Ok, Err> {

    // }
}

pub trait ListComponents {
    fn list(self) -> Vec<Box<dyn Component>>;
}


macro_rules! impl_list_components {
    ($($comp: ident),*) => {
        impl<$($comp),*> ListComponents for ($($comp,)*)
        where
            $(
                $comp: Component,
            )*
        {

            #[allow(non_snake_case)]
            fn list(self) -> Vec<Box<dyn Component>> {
                let mut result: Vec<Box<dyn Component>> = Vec::new();

                let ($($comp,)*) = self;
                $(
                    result.push(Box::new($comp));
                )*
                result
            }
        }
    }
}

recursive!(impl_list_components, A, B, C, D);


impl Archetype {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: HashMap::new(),
        }
    }
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




/*
#[macro_export]
macro_rules! expand {
    ($m: ident, $ty: ident) => {
        $m!{$ty}
    };
    ($m: ident, $ty: ident, $($tt: ident),*) => {
        $m!{$ty, $($tt),*}
        expand!{$m, $($tt),*}
    };
}
*/
