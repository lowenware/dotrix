//! Voxel Module
//!
//! Handles general voxel related content, such as conversion to an explicit
//! mesh using marching cubes or direct rendering.
//!

mod grid;
mod voxel;

pub use grid::Grid;
pub use voxel::Voxel;
