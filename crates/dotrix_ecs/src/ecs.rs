use crate::world::World;

/// Entity structure has only id field and represent an agregation of components
pub struct Entity(u64);

/// Any data structure can be a component
pub trait Component: Send + Sync + 'static {
}

impl<T: Send + Sync + 'static> Component for T {}

/// Trait for ECS systems
pub trait System {
    fn startup(&mut self, world: &mut World);
    fn run_cycle(&mut self, world: &mut World);
}

