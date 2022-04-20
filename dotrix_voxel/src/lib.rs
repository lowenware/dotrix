//! Voxel Module
//!
//! Handles general voxel related content, such as conversion to an explicit
//! mesh using marching cubes or direct rendering.
//!

use dotrix_core::Application;

mod grid;
mod material_set;
mod sdf;
mod voxel;

pub use grid::Grid;
pub use material_set::*;
pub use sdf::*;
pub use voxel::Voxel;

/// Enables Voxel Dotrix Extension
pub fn extension(app: &mut Application) {
    sdf::extension(app);
}
