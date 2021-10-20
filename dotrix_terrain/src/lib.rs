//! Dotrix terrain implementation
#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

use std::any::Any;

use dotrix_core::{ Application, Id, System };
use dotrix_core::assets::Mesh;
use dotrix_core::ray::Ray;
use dotrix_math::Vec3;

// mod generator;
mod height_map;
mod layers;
mod map;
mod systems;
mod simple;

// pub use noise::{ Noise };
pub use height_map::HeightMap;
pub use layers::{ Layers, Layer };
pub use map::{ Component, Lod, Map, Node, Noise, VecXZ };
pub use systems::{ startup, render, spawn };
pub use simple::Simple;


/// Terrain tile component
pub struct Terrain {
    /// Terrain position
    pub position: VecXZ<i32>,
    /// Terrain scale
    pub scale: u32,
    /// Terrain mesh ID
    pub mesh: Id<Mesh>,
    /// True if it was loaded to GPU
    pub loaded: bool,
}

pub trait Generator: Send + Sync {
    fn get(
        &self,
        component: Component,
        position: VecXZ<i32>,
        scale: u32,
        unit_size: f32
    ) -> Option<Mesh>;

    fn dirty(&self) -> bool;

    fn set_dirty(&mut self, value: bool);

    fn intersection(&self, ray: &Ray, range: f32, unit_size: f32) -> Option<Vec3>;

    fn modify(&mut self, point: &Vec3, values: &[f32], size: u32, unit_size: f32);

    fn flatten(&mut self, point: &Vec3, values: &[f32], size: u32, unit_size: f32);
}

/// Enables the terrain extension in Dotrix application
pub fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(spawn));
    app.add_system(System::from(render));
    app.add_service(Map::default());
}
