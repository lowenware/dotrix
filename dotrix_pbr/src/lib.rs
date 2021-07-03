//! Physically based rendering implementation

use dotrix_core::Application;
use dotrix_core::ecs::System;

mod light;
mod material;
mod model;

/// Solid models rendering
pub mod solid;

/// Skeletal models rendering
pub mod skeletal;

pub use light::{ Light, Lights };
pub use material::Material;
pub use model::Model;

/// Enables PBR Dotrix Extension
pub fn extension(app: &mut Application) {
    app.add_system(System::from(light::startup));
    app.add_system(System::from(light::bind));

    solid::extension(app);
    skeletal::extension(app);
}
