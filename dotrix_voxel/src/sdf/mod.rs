use dotrix_core::ecs::System;
use dotrix_core::Application;

mod jump_flood;
mod tex_sdf;

pub use jump_flood::*;
pub use tex_sdf::*;

/// Enables Voxel SDF Dotrix Extension
pub fn extension(app: &mut Application) {
    app.add_system(System::from(jump_flood::startup));
    app.add_system(System::from(jump_flood::compute));
}
