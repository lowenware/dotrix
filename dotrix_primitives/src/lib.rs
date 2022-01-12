//! Collection of dotrix primitives

mod cube;

pub use cube::Cube;

/// Additional mesh attributes
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum MeshAttribute {
    /// Positions
    Positions,
    /// Normals to surfaces
    Normals,
    /// Tangents Bitangents
    TangentsBitangents,
    /// texture positions
    UVs,
}
