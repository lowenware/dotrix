//! Traits that provide certain features
//!

use crate::renderer::{AttributeFormat, Buffer as GpuBuffer, Texture as GpuTexture};

/// Provides a GPU buffer
pub trait BufferProvider {
    /// Get the underlying gpu buffer
    fn get_buffer(&self) -> &GpuBuffer;

    /// Get the underlying gpu buffer mutable
    fn get_buffer_mut(&mut self) -> &mut GpuBuffer;
}

/// Provides a GPU Texture
pub trait TextureProvider {
    /// Get the underlying gpu texture
    fn get_texture(&self) -> &GpuTexture;

    /// Get the underlying gpu texture mutable
    fn get_texture_mut(&mut self) -> &mut GpuTexture;
}

/// Provides mesh buffers and data
pub trait MeshProvider {
    /// Get the underlying vertex buffer
    fn get_vertex(&self) -> &GpuBuffer;
    /// Get the underlying optional index buffer
    fn get_indicies(&self) -> Option<&GpuBuffer>;

    /// Get the underlying vertex buffer mutable
    fn get_vertex_mut(&mut self) -> &mut GpuBuffer;
    /// Get the underlying optional index buffer mutable
    fn get_indicies_mut(&mut self) -> Option<&mut GpuBuffer>;

    /// Get the number of verticies
    fn get_vertex_count(&self) -> u32;

    /// Get the layout of a vertex
    fn get_vertex_buffer_layout(&self) -> &[AttributeFormat];
}
