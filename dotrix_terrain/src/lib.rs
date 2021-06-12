//! Dotrix terrain implementation
#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

mod heightmap;
mod services;
mod systems;

pub use heightmap::{ Heightmap, Generator };
pub use services::Manager;
pub use systems::spawn;

/// Terrain chunk (tile) component
pub struct Terrain {
    /// Terrain position by X axis (center of the chunk)
    pub x: i32,
    /// Terrain position by Z axis (center of the chunk)
    pub z: i32,
    /// Terrain chunk level of details (0 is the highest)
    pub lod: usize,
}

