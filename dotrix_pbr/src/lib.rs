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

pub fn add_pbr_to_shader(source: &str, bind_group: usize, binding: usize) -> String {
    let pbr_code = include_str!("shaders/pbr.inc.wgsl");

    let pbr_lighted_code = Lights::add_to_shader(pbr_code, bind_group, binding);

    source.replace("{{ include(light) }}", &pbr_lighted_code)
}
