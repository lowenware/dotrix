use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::graphics::Format;
use crate::loaders::Asset;
use crate::math::{Vec2, Vec3};

use super::{
    VertexAttribute, VertexBitangent, VertexNormal, VertexPosition, VertexTangent, VertexTexture,
};

/// 3D Model Mesh
pub struct Mesh {
    name: String,
    vertices: HashMap<TypeId, AttributeValues>,
    vertices_count: usize,
    indices: Option<Vec<u32>>,
}

impl Mesh {
    /// Constructs new Mesh instance
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            vertices: HashMap::new(),
            vertices_count: 0,
            indices: None,
        }
    }

    /// Constructs a cube mesh
    pub fn cube(name: impl ToString) -> Self {
        Mesh::generate(name, genmesh::generators::Cube::new())
    }

    /// Constructs a cylinder mesh
    /// `u` is the number of points across the radius
    ///
    /// `h` is the number of segments across the height
    pub fn cylinder(name: impl ToString, u: usize, h: Option<usize>) -> Self {
        Mesh::generate(
            name,
            h.map_or_else(
                || genmesh::generators::Cylinder::new(u),
                |h| genmesh::generators::Cylinder::subdivide(u, h),
            ),
        )
    }

    /// Constructs a sphere mesh
    ///
    /// `u` is the number of points across the equator of the sphere
    ///
    /// `v` is the number of points from pole to pole
    pub fn sphere(name: impl ToString, u: usize, v: usize) -> Self {
        Mesh::generate(name, genmesh::generators::SphereUv::new(u, v))
    }

    /// Constructs a cone mesh
    ///
    /// `u` is the number of subdivisions around the radius and it must be greater then 1
    pub fn cone(name: impl ToString, u: usize) -> Self {
        Mesh::generate(name, genmesh::generators::Cone::new(u))
    }

    /// Constructs a torus mesh
    ///
    /// `radius` is the radius from the center [0, 0, 0] to the center of the the tubular
    /// radius
    ///
    /// `tubular_radius is the radius to the surface from the toridal
    ///
    /// `tubular_segments` is the number of segments that wrap around the tube, must be at least 3
    ///
    /// `radial_segments` is  the number of tube segments requested to generate, must be at least 3
    pub fn torus(
        name: impl ToString,
        radius: f32,
        tubular_radius: f32,
        radial_segments: usize,
        tubular_segments: usize,
    ) -> Self {
        Mesh::generate(
            name,
            genmesh::generators::Torus::new(
                radius,
                tubular_radius,
                radial_segments,
                tubular_segments,
            ),
        )
    }

    fn generate<G, I, P>(name: impl ToString, generator: G) -> Self
    where
        I: genmesh::EmitTriangles<Vertex = genmesh::Vertex>,
        I::Vertex: Clone + Copy + PartialEq,
        P: genmesh::EmitTriangles<Vertex = usize>,
        G: genmesh::generators::SharedVertex<I::Vertex>
            + genmesh::generators::IndexedPolygon<P>
            + Iterator<Item = I>,
    {
        use genmesh::{MapVertex, Triangulate, Vertices};

        let mut mesh = Mesh::new(name);
        let vertices = generator.shared_vertex_iter().collect::<Vec<_>>();
        let vertices_count = vertices.len();
        let mut positions = Vec::with_capacity(vertices_count);
        let mut normals = Vec::with_capacity(vertices_count);
        let mut tex_uvs = Vec::with_capacity(vertices_count);

        for (position, normal, tex_uv) in generator
            .indexed_polygon_iter()
            .triangulate()
            .map(|triangle| {
                triangle.map_vertex(|v| {
                    let vertex = vertices[v];
                    let position = [vertex.pos.x, vertex.pos.y, vertex.pos.z];
                    let normal = [vertex.normal.x, vertex.normal.y, vertex.normal.z];
                    let tex_uv = [(vertex.pos.x + 1.0) / 2.0, (vertex.pos.y + 1.0) / 2.0];
                    (position, normal, tex_uv)
                })
            })
            .vertices()
        {
            positions.push(position);
            normals.push(normal);
            tex_uvs.push(tex_uv);
        }

        mesh.set_vertices::<VertexPosition>(positions);
        mesh.set_vertices::<VertexNormal>(normals);
        mesh.set_vertices::<VertexTexture>(tex_uvs);

        mesh
    }

    /// Sets vertices attributes by Type
    pub fn set_vertices<A: VertexAttribute>(&mut self, values: Vec<A::Raw>) {
        let vertices_count = values.len();

        // assert vertices count
        if self.vertices_count != vertices_count {
            if self.vertices_count != 0 {
                panic!(
                    "Mesh '{}' has {} vertices, but attribute '{}' was given with {} values.",
                    self.name,
                    self.vertices_count,
                    A::name(),
                    vertices_count
                );
            }
            self.vertices_count = vertices_count;
        }

        let format = A::format();
        let attribute_size = format.size();
        let values_len = values.len() * attribute_size;
        let values_capacity = values.capacity() * attribute_size;
        let bytes: Vec<u8> = unsafe {
            Vec::from_raw_parts(
                bytemuck::cast_slice_mut::<_, u8>(values.leak()) as *mut [u8] as *mut u8,
                values_len,
                values_capacity,
            )
        };

        // store attributes
        self.vertices
            .insert(TypeId::of::<A>(), AttributeValues { format, bytes });
    }

    /// Returns slice of vertices attributes if exists
    pub fn vertices<A: VertexAttribute>(&self) -> Option<&[A::Raw]> {
        self.vertices
            .get(&TypeId::of::<A>())
            .map(|values| bytemuck::cast_slice(&values.bytes))
    }

    /// Sets mesh indices
    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.indices = Some(indices);
    }

    /// Clears mesh indices
    pub fn clear_indices(&mut self) {
        self.indices = None;
    }

    /// Returns type casted list of indices
    ///
    /// Use u32 to get indices themselves or u8 to get data for buffering
    pub fn indices<T: bytemuck::Pod + bytemuck::Zeroable>(&self) -> Option<&[T]> {
        self.indices.as_ref().map(|i| bytemuck::cast_slice(i))
    }

    /// Returns number of vertices
    pub fn count_vertices(&self) -> usize {
        self.vertices_count
    }

    /// Returns vector of vertex buffer data according to layout defined by attributes types
    pub fn buffer<T: VertexBufferLayout>(&self) -> Option<Vec<u8>> {
        self.buffer_from_layout(&T::layout())
    }

    pub fn contains<T: VertexBufferLayout>(&self) -> bool {
        self.has_layout(&T::layout())
    }

    pub fn has_layout(&self, layout: &[TypeId]) -> bool {
        for t in layout.iter() {
            if !self.vertices.contains_key(t) {
                return false;
            }
        }
        true
    }

    /// Returns vector of vertex buffer data according to layout
    pub fn buffer_from_layout(&self, layout: &[TypeId]) -> Option<Vec<u8>> {
        if !self.has_layout(layout) {
            return None;
        }
        let buffer = (0..self.vertices_count)
            .map(|i| {
                layout
                    .iter()
                    .map(move |t| {
                        let values = self.vertices.get(t).unwrap();
                        let size = values.format.size();
                        let offset = i * size;
                        let cut = offset + size;
                        &values.bytes[offset..cut]
                    })
                    .flatten()
            })
            .flatten()
            .map(|v| *v)
            .collect::<Vec<u8>>();
        return Some(buffer);
    }

    /// Returns number of faces (polygons) in the mesh
    pub fn count_faces(&self) -> usize {
        self.indices
            .as_ref()
            .map(|i| i.len())
            .unwrap_or(self.vertices_count)
            / 3
    }

    /// Calculates normals for the Mesh
    pub fn calculate_normals(&self) -> Option<Vec<[f32; 3]>> {
        self.vertices::<VertexPosition>().map(|positions| {
            let mut normals = vec![[99.9; 3]; self.vertices_count];
            let faces = self.count_faces();
            for face in 0..faces {
                let mut i0 = (face * 3) as usize;
                let mut i1 = i0 + 1;
                let mut i2 = i1 + 1;
                if let Some(indices) = self.indices.as_ref() {
                    i0 = indices[i0] as usize;
                    i1 = indices[i1] as usize;
                    i2 = indices[i2] as usize;
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
        })
    }

    /// Calculates normals for the mesh and stores them
    pub fn auto_normals(&mut self) {
        if self.vertices.contains_key(&TypeId::of::<VertexNormal>()) {
            return;
        }
        if let Some(normals) = self.calculate_normals() {
            self.set_vertices::<VertexNormal>(normals);
        }
    }

    /// Calculates tangents for the mesh
    pub fn calculate_tangents_bitangents(&self) -> Option<(Vec<[f32; 3]>, Vec<[f32; 3]>)> {
        self.vertices::<VertexPosition>()
            .zip(self.vertices::<VertexTexture>())
            .map(|(positions, uvs)| {
                let mut tangents = vec![[99.9; 3]; self.vertices_count];
                let mut bitangents = vec![[99.9; 3]; self.vertices_count];
                let faces = self.count_faces();
                for face in 0..faces {
                    let mut i0 = (face * 3) as usize;
                    let mut i1 = i0 + 1;
                    let mut i2 = i1 + 1;
                    if let Some(indices) = self.indices.as_ref() {
                        i0 = indices[i0] as usize;
                        i1 = indices[i1] as usize;
                        i2 = indices[i2] as usize;
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
            })
    }

    /// Calculates tangents for the mesh and stores them
    pub fn auto_tangents_bitangents(&mut self) {
        let has_tangents = self.vertices.contains_key(&TypeId::of::<VertexTangent>());
        let has_bitangents = self.vertices.contains_key(&TypeId::of::<VertexBitangent>());
        if has_tangents && has_bitangents {
            return;
        }
        if let Some((tangents, bitangents)) = self.calculate_tangents_bitangents() {
            self.set_vertices::<VertexTangent>(tangents);
            self.set_vertices::<VertexBitangent>(bitangents);
        }
    }
}

pub struct AttributeValues {
    format: Format,
    bytes: Vec<u8>,
}

pub trait VertexBufferLayout {
    type Layout;
    fn layout() -> Vec<TypeId>;
    fn vertex_size() -> usize;
    fn attributes() -> VertexAttributeIter<Self::Layout>;
}

pub struct VertexAttributeIter<T> {
    index: u32,
    offset: u64,
    _phantom_data: PhantomData<T>,
}

pub type VertexAttributeIterItem = (Format, u64, u32);

macro_rules! impl_layout {
    (($($i: ident),*)) => {
        impl<$($i,)*> VertexBufferLayout for ($($i,)*)
        where
            $($i: VertexAttribute,)*
        {
            type Layout = ($($i,)*);

            fn layout() -> Vec<TypeId> {
                vec![
                    $(TypeId::of::<$i>(),)*
                ]
            }

            fn vertex_size() -> usize {
                0 $(+ ($i::format()).size())*
            }

            fn attributes() -> VertexAttributeIter<Self::Layout> {
                VertexAttributeIter {
                    index: 0,
                    offset: 0,
                    _phantom_data: PhantomData
                }
            }
        }

        impl<$($i,)*> Iterator for VertexAttributeIter<($($i,)*)>
        where
            $($i: VertexAttribute,)*
        {
            type Item = VertexAttributeIterItem;
            fn next(&mut self) -> Option<VertexAttributeIterItem> {
                let formats = [ $($i::format(),)* ];
                let index = self.index;
                let offset = self.offset;
                let next_index = index + 1;

                if index < formats.len() as u32 {
                    let format = formats[index as usize];
                    self.index = next_index;
                    self.offset += format.size() as u64;

                    Some((format, offset, index))
                } else {
                    None
                }
            }
        }

    }
}

impl_layout!((A));
impl_layout!((A, B));
impl_layout!((A, B, C));
impl_layout!((A, B, C, D));
impl_layout!((A, B, C, D, E));
impl_layout!((A, B, C, D, E, F));
impl_layout!((A, B, C, D, E, F, G));
impl_layout!((A, B, C, D, E, F, G, H));
impl_layout!((A, B, C, D, E, F, G, H, I));
impl_layout!((A, B, C, D, E, F, G, H, I, J));

impl Asset for Mesh {
    fn name(&self) -> &str {
        &self.name
    }
}
