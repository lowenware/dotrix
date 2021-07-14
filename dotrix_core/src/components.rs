//! Component usually is just a data structure with or without associated methods. In ECS
//! pattern components represent properties of entities: velocity, weight, model, rigid body, color
//! etc.
//!
//! Set of components in the entity defines an Archetype. Entities of the same
//! Archetype are being grouped together in the [`crate::services::World`] storage, that
//! makes search fast and linear.
//!
//! When planning the architecture of your game, developer should think about components not only
//! as of properties, but also as of search tags. For example, if you are making physics for your
//! game, and you have a `Car` and a `SpringBoard`, you may want to have the same named components,
//! so you can easily query all `Cars` or all `SpringBoards`. But as soon as you will need to
//! calculate physics for entities of the both types, you should add some component like 
//! `RigidBody` to the entities, so you can calculate physics for all entities who have that
//! component.
//!
//! ## Usefull references
//! - To learn how to spawn and query entities, continue reading with [`crate::services::World`]
//! - To learn how to implement systems [`crate::systems`]

mod light;
mod material;
mod model;
mod pose;
mod pipeline;
mod transform;

// TODO: need another modules structure
pub use { light::Light, light::Lights, light::load as load_lights, light::startup as startup_lights };
pub use material::Material;
pub use model::Model;
pub use pose::Pose;
pub use pipeline::Pipeline;
pub use transform::Transform;

// TODO: consider moving
pub use crate::animation::Animator;
