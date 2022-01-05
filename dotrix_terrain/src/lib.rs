//! Dotrix terrain implementation
#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

use std::any::Any;

use dotrix_core::assets::Mesh;
use dotrix_core::{Application, Id, System};

mod generator;
mod layers;
mod services;
mod systems;

pub use generator::{Falloff, Generator, Noise};
pub use layers::{Layer, Layers};
pub use services::Terrain;
pub use systems::{render, spawn, startup};

/// Terrain tile component
pub struct Tile {
    /// Terrain position by X axis (center of the chunk)
    pub x: i32,
    /// Terrain position by Z axis (center of the chunk)
    pub z: i32,
    /// Terrain chunk level of details (0 is the highest)
    pub lod: usize,
    /// Terrain chunk mesh ID
    pub mesh: Id<Mesh>,
    /// Is loaded by GPU
    pub loaded: bool,
}

/// Trait for the terrain heights source
pub trait Heightmap: Any + Sync + Send {
    /// Returns Y axis value for specified X and Z pair
    fn value(&self, x: usize, z: usize) -> f32;
    /// Returns number of values per map side
    fn size(&self) -> usize;
}

impl dyn Heightmap {
    /// Casts down the reference
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&*(self as *const dyn Heightmap as *const T)) }
        } else {
            None
        }
    }

    /// Casts down the mutable reference
    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&mut *(self as *mut dyn Heightmap as *mut T)) }
        } else {
            None
        }
    }

    /// Checks if the reference is of specific type
    #[inline]
    fn is<T: Any>(&self) -> bool {
        std::any::TypeId::of::<T>() == self.type_id()
    }
}

/// Enables the terrain extension in Dotrix application
pub fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(spawn));
    app.add_system(System::from(render));
    app.add_service(Terrain::default());
}
