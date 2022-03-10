use dotrix_core::ecs::System;
use dotrix_core::Application;

pub mod jump_flood;

pub use jump_flood::*;

/// Enables Voxel SDF Dotrix Extension
pub fn extension(app: &mut Application) {
    app.add_system(System::from(jump_flood::startup));
    app.add_system(System::from(jump_flood::compute));
}
