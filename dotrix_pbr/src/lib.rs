//! Physically based rendering implementation

use dotrix_core::ecs::System;
use dotrix_core::Application;

mod light;
mod material;
mod model;

/// Solid models rendering
pub mod solid;

/// Skeletal models rendering
pub mod skeletal;

pub use light::{Light, Lights};
pub use material::Material;
pub use model::Model;

/// Enables PBR Dotrix Extension
pub fn extension(app: &mut Application) {
    app.add_system(System::from(material::startup));
    app.add_system(System::from(light::startup));
    app.add_system(System::from(light::load));

    solid::extension(app);
    skeletal::extension(app);
}
