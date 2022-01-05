//! Mesh Asset
use crate::renderer::{AttributeFormat, Renderer, VertexBuffer};
use bytemuck::{Pod, Zeroable};
use dotrix_math::{InnerSpace, Vec3, VectorSpace};

/// Asset with 3D model data
#[derive(Default)]
pub struct Mesh {
    /// Packed array of vertices data
    pub vertices: Vec<Vec<u8>>,
    /// Size of all Vertex Attributes in bytes
    pub stride: usize,
    /// Vertex buffer layout
    pub layout: Vec<AttributeFormat>,
    /// Optional indices
    pub indices: Option<Vec<u8>>,
    /// vertex buffer instance
    pub vertex_buffer: VertexBuffer,
    /// Flag to react on the mesh changes
    pub changed: bool,
}

impl Mesh {
    /// Adds Vertex Attributes to the mesh
    pub fn with_vertices<T>(&mut self, data: &[T])
    where
        T: VertexAttribute + Pod + Zeroable,
    {
        if self.vertices.is_empty() {
            self.vertices = data
                .iter()
                .map(|attr| Vec::from(bytemuck::cast_slice(&[*attr])))
                .collect::<Vec<_>>();
        } else {
            if self.vertices.len() != data.len() {
                panic!("Arrays of vertices attributes should have the same size");
            }
            for (vertex_data, new_attr) in self.vertices.iter_mut().zip(data.iter()) {
                vertex_data.extend(bytemuck::cast_slice(&[*new_attr]));
            }
        }
        let format = T::format();
        self.stride += format.size();
        self.layout.push(format);
    }

    /// Sets indicies to the mesh
    pub fn with_indices(&mut self, indices: &[u32]) {
        self.indices = Some(Vec::from(bytemuck::cast_slice(indices)));
    }

    /// Load the [`Mesh`] buffer
    pub fn load(&mut self, renderer: &Renderer) {
        if !self.changed && !self.vertex_buffer.is_empty() {
            return;
        }

        let count = self
            .indices
            .as_ref()
            .map(|indices| indices.len() / 4)
            .unwrap_or_else(|| self.vertices.len());

        let buffer: Vec<u8> = self.vertices.iter().flatten().copied().collect::<Vec<_>>();

        renderer.load_vertex_buffer(
            &mut self.vertex_buffer,
            buffer.as_slice(),
            self.indices.as_deref(),
            count,
        );

        self.changed = false;
    }

    /// Unloads the [`Mesh`] buffer
    pub fn unload(&mut self) {
        self.vertex_buffer.empty();
    }

    /// Returns actual mesh vertex buffer layout
    pub fn vertex_buffer_layout(&self) -> &[AttributeFormat] {
        &self.layout
    }

    /// Calculates normals for the mesh
    pub fn calculate_normals(positions: &[[f32; 3]], indices: Option<&[u32]>) -> Vec<[f32; 3]> {
        let mut normals = vec![[99.9; 3]; positions.len()];
        let faces = indices.map(|i| i.len()).unwrap_or_else(|| positions.len()) / 3;

        for face in 0..faces {
            let mut i0 = (face * 3) as usize;
            let mut i1 = i0 + 1;
            let mut i2 = i1 + 1;
            if let Some(idx) = indices {
                i0 = idx[i0] as usize;
                i1 = idx[i1] as usize;
                i2 = idx[i2] as usize;
            }
            let v0 = Vec3::from(positions[i0]);
            let v1 = Vec3::from(positions[i1]);
            let v2 = Vec3::from(positions[i2]);
            let n = (v1 - v0).cross(v2 - v1).normalize();
            normals[i0] = if normals[i0][0] > 9.0 {
                n.into()
            } else {
                n.lerp(normals[i0].into(), 0.5).into()
            };
            normals[i1] = if normals[i1][0] > 9.0 {
                n.into()
            } else {
                n.lerp(normals[i1].into(), 0.5).into()
            };
            normals[i2] = if normals[i2][0] > 9.0 {
                n.into()
            } else {
                n.lerp(normals[i2].into(), 0.5).into()
            };
        }
        normals
    }
}

/// Vertex attribute abstraction
pub trait VertexAttribute {
    /// Returns attribute format
    fn format() -> AttributeFormat;
}

impl VertexAttribute for f32 {
    fn format() -> AttributeFormat {
        AttributeFormat::Float32
    }
}

impl VertexAttribute for [f32; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float32x2
    }
}

impl VertexAttribute for [f32; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float32x3
    }
}

impl VertexAttribute for [f32; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float32x4
    }
}

impl VertexAttribute for [u16; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Uint16x2
    }
}

impl VertexAttribute for [u16; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Uint16x4
    }
}

impl VertexAttribute for u32 {
    fn format() -> AttributeFormat {
        AttributeFormat::Uint32
    }
}

impl VertexAttribute for [u32; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Uint32x2
    }
}

impl VertexAttribute for [u32; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Uint32x3
    }
}

impl VertexAttribute for [u32; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Uint32x4
    }
}
