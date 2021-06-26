//! Dotrix terrain implementation
#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

use std::any::Any;

use dotrix_core::renderer::Color;

mod generator;
mod services;
mod systems;
mod pipeline;

pub use generator::{ Falloff, Generator, Noise };
pub use services::Manager;
pub use systems::{ startup, spawn };
pub use pipeline::new_pipeline;

/// Terrain chunk (tile) component
pub struct Terrain {
    /// Terrain position by X axis (center of the chunk)
    pub x: i32,
    /// Terrain position by Z axis (center of the chunk)
    pub z: i32,
    /// Terrain chunk level of details (0 is the highest)
    pub lod: usize,
}

/// Terrain layer
pub struct Layer {
    /// Terrain layer color
    pub color: Color,
    /// Terrain layer height base 0.0..1.0
    pub base: f32,
    /// Terrain layer blend
    pub blend: f32,
}

impl Default for Layer {
    fn default() -> Self {
        Self { color: Color::rgb(0.18, 0.62, 0.24), base: -1.0, blend: 0.1 }
    }
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
