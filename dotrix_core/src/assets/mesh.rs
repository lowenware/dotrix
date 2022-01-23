//! Mesh Asset
use crate::renderer::{AttributeFormat, Renderer, VertexBuffer};
use bytemuck::{Pod, Zeroable};
use dotrix_math::{InnerSpace, Vec2, Vec3, VectorSpace};
use std::marker::PhantomData;

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

    /// Get vertices with type casting
    pub fn vertices_as<T>(&self, index: usize) -> AttributeIter<T>
    where
        T: VertexAttribute + Pod + Zeroable,
    {
        let mut offset = 0;
        for i in 0..index {
            offset += self.layout[i].size();
        }
        let size = offset + self.layout[index].size();

        if self.layout[index].type_id() != std::any::TypeId::of::<T>() {
            panic!("Wrong attribute type");
        }

        AttributeIter {
            iter: self.vertices.iter(),
            offset,
            size,
            _phantom: PhantomData,
        }
    }

    /// Sets indices to the mesh
    pub fn with_indices(&mut self, indices: &[u32]) {
        self.indices = Some(Vec::from(bytemuck::cast_slice(indices)));
    }

    /// Get vertices with type casting
    pub fn indices(&self) -> Option<&[u32]> {
        self.indices
            .as_ref()
            .map(|v| bytemuck::cast_slice(v.as_slice()))
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

    /// Calculates tangents for the mesh
    pub fn calculate_tangents_bitangents(
        positions: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: Option<&[u32]>,
    ) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let mut tangents = vec![[99.9; 3]; positions.len()];
        let mut bitangents = vec![[99.9; 3]; positions.len()];
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
            let uv0 = Vec2::from(uvs[i0]);
            let uv1 = Vec2::from(uvs[i1]);
            let uv2 = Vec2::from(uvs[i2]);

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

            let tangent = Vec3::from([
                f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
                f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
                f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
            ]);

            let bitangent = Vec3::from([
                f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x),
                f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y),
                f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z),
            ]);

            tangents[i0] = if tangents[i0][0] > 9.0 {
                tangent.into()
            } else {
                tangent.lerp(tangents[i0].into(), 0.5).into()
            };
            tangents[i1] = if tangents[i1][0] > 9.0 {
                tangent.into()
            } else {
                tangent.lerp(tangents[i1].into(), 0.5).into()
            };
            tangents[i2] = if tangents[i2][0] > 9.0 {
                tangent.into()
            } else {
                tangent.lerp(tangents[i2].into(), 0.5).into()
            };

            bitangents[i0] = if bitangents[i0][0] > 9.0 {
                bitangent.into()
            } else {
                bitangent.lerp(bitangents[i0].into(), 0.5).into()
            };
            bitangents[i1] = if bitangents[i1][0] > 9.0 {
                bitangent.into()
            } else {
                bitangent.lerp(bitangents[i1].into(), 0.5).into()
            };
            bitangents[i2] = if bitangents[i2][0] > 9.0 {
                bitangent.into()
            } else {
                bitangent.lerp(bitangents[i2].into(), 0.5).into()
            };
        }

        (tangents, bitangents)
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

/// Iterator over Vertices Attributes
pub struct AttributeIter<'a, T> {
    iter: std::slice::Iter<'a, Vec<u8>>,
    offset: usize,
    size: usize,
    _phantom: PhantomData<T>,
}

impl<'a, T> Iterator for AttributeIter<'a, T>
where
    T: VertexAttribute + Pod + Zeroable,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|v| bytemuck::cast_slice::<u8, T>(&v[self.offset..self.size])[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertices_as() {
        let mut width = 2.0;

        let verticies_test_original_1: Vec<[f32; 3]> = vec![
            [-width, -width, -width],
            [width, -width, -width],
            [width, width, -width],
            [-width, width, -width],
            [-width, -width, width],
            [width, -width, width],
            [width, width, width],
            [-width, width, width],
        ];

        width = width / 2.0;

        let verticies_test_original_2: Vec<[f32; 3]> = vec![
            [-width, -width, -width],
            [width, -width, -width],
            [width, width, -width],
            [-width, width, -width],
            [-width, -width, width],
            [width, -width, width],
            [width, width, width],
            [-width, width, width],
        ];

        width = width / 2.0;

        let verticies_test_original_3: Vec<[u32; 3]> = vec![
            [-width as u32, -width as u32, -width as u32],
            [width as u32, -width as u32, -width as u32],
            [width as u32, width as u32, -width as u32],
            [-width as u32, width as u32, -width as u32],
            [-width as u32, -width as u32, width as u32],
            [width as u32, -width as u32, width as u32],
            [width as u32, width as u32, width as u32],
            [-width as u32, width as u32, width as u32],
        ];

        let indices_test_original = [
            0, 2, 1, 0, 3, 2, 1, 6, 5, 1, 2, 6, 5, 7, 4, 5, 6, 7, 4, 3, 0, 4, 7, 3, 3, 6, 2, 3, 7,
            6, 4, 1, 5, 4, 0, 1,
        ];

        let mut mesh = Mesh::default();

        mesh.with_vertices(&verticies_test_original_1);
        mesh.with_vertices(&verticies_test_original_2);
        mesh.with_vertices(&verticies_test_original_3);
        mesh.with_indices(&indices_test_original);

        let verticies_test_1 = mesh.vertices_as::<[f32; 3]>(0).collect::<Vec<_>>();
        let verticies_test_2 = mesh.vertices_as::<[f32; 3]>(1).collect::<Vec<_>>();
        let verticies_test_3 = mesh.vertices_as::<[u32; 3]>(2).collect::<Vec<_>>();
        let indices_test = mesh.indices().unwrap();

        assert_eq!(verticies_test_original_1, verticies_test_1);
        assert_eq!(verticies_test_original_2, verticies_test_2);
        assert_eq!(verticies_test_original_3, verticies_test_3);
        assert_eq!(&indices_test_original, indices_test);
    }

    #[test]
    #[should_panic]
    fn test_vertices_as_typecheck() {
        let width = 2.0;

        let verticies_test_original_1: Vec<[f32; 3]> = vec![
            [-width, -width, -width],
            [width, -width, -width],
            [width, width, -width],
            [-width, width, -width],
            [-width, -width, width],
            [width, -width, width],
            [width, width, width],
            [-width, width, width],
        ];

        let mut mesh = Mesh::default();

        mesh.with_vertices(&verticies_test_original_1);

        mesh.vertices_as::<[u32; 3]>(0).for_each(drop);
    }
}
