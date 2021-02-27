use bytemuck::{ Pod, Zeroable };
use wgpu::util::DeviceExt;
use dotrix_math::{ Vec3, InnerSpace, VectorSpace };

/// Asset with 3D model data
#[derive(Default)]
pub struct Mesh {
    /// Vertices positions
    pub positions: Vec<[f32; 3]>,
    /// Normals for the vertices
    pub normals: Option<Vec<[f32; 3]>>,
    /// Texture coordinates for the vertices
    pub uvs: Option<Vec<[f32; 2]>>,
    /// Transformation weights for the vertices
    pub weights: Option<Vec<[f32; 4]>>,
    /// Indices of joints affecting the transformation
    pub joints: Option<Vec<[u16; 4]>>,
    /// Indices of the vertices
    pub indices: Option<Vec<u32>>,
    /// Vertices pipeline buffer
    pub vertices_buffer: Option<wgpu::Buffer>,
    /// Indices pipeline buffer
    pub indices_buffer: Option<wgpu::Buffer>,
}

impl Mesh {
    /// Converts a mesh into a vector of vertices data, packed for static shadering
    /// (positions, normals, uvs)
    pub fn as_static(&self) -> Option<Vec<StaticModelVertex>> {
        if let Some(normals) = self.normals.as_ref() {
            if let Some(uvs) = self.uvs.as_ref() {
                return Some(
                    self.positions
                        .iter()
                        .zip(normals.iter().zip(uvs.iter()))
                        .map(|(position, (normal, uv))| {
                            StaticModelVertex {
                                position: *position,
                                normal: *normal,
                                uv: *uv,
                            }
                        })
                        .collect::<Vec<_>>()
                );
            }
        }
        None
    }

    /// Converts a mesh into a vector of vertices data, packed for skinned shadering
    /// (positions, normals, uvs, weights, affected joints)
    pub fn as_skinned(&self) -> Option<Vec<SkinnedModelVertex>> {
        if let Some(normals) = self.normals.as_ref() {
            if let Some(uvs) = self.uvs.as_ref() {
                if let Some(all_weights) = self.weights.as_ref() {
                    if let Some(all_joints) = self.joints.as_ref() {
                        let weights_joints = all_weights.iter().zip(all_joints.iter());
                        return Some(
                            self.positions
                                .iter()
                                .zip(normals.iter().zip(uvs.iter().zip(weights_joints)))
                                .map(|(position, (normal, (uv, (weights, joints))))| {
                                    SkinnedModelVertex {
                                        position: *position,
                                        normal: *normal,
                                        uv: *uv,
                                        weights: *weights,
                                        joints: *joints,
                                    }
                                })
                                .collect::<Vec<_>>()
                        );
                    }
                }
            }
        }
        None
    }

    /// Checks if the [`Mesh`] has information about [`crate::assets::Skin`]
    pub fn is_skinned(&self) -> bool {
        self.weights.is_some() && self.joints.is_some()
    }

    /// Returns the number of the [`Mesh`] indices
    pub fn indices_count(&self) -> u32 {
        self.indices
            .as_ref()
            .map(|i| i.len())
            .unwrap_or_else(|| self.positions.len()) as u32
    }

    /// Generates a cube [`Mesh`]
    pub fn cube() -> Self {
        Self {
            positions: vec!(
                // front
                [-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0],
                // top 
                [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0], [-1.0, 1.0, 1.0], [1.0, 1.0, 1.0],
                // right
                [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [1.0, 1.0, 1.0], [1.0, -1.0, 1.0],
                // left
                [-1.0, -1.0, 1.0], [-1.0, 1.0, 1.0], [-1.0, 1.0, -1.0], [-1.0, -1.0, -1.0],
                // back
                [-1.0, 1.0, -1.0], [1.0, 1.0, -1.0], [1.0, -1.0, -1.0], [-1.0, -1.0, -1.0],
                // bottom
                [1.0, -1.0, 1.0], [-1.0, -1.0, 1.0], [-1.0, -1.0, -1.0], [1.0, -1.0, -1.0],
            ),
            normals: Some(vec!(
                // front
                [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
                // top 
                [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
                // right
                [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
                // left
                [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
                // back
                [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
                // bottom
                [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
            )),
            uvs: Some(vec!(
                // front
                [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
                // top 
                [1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0],
                // right
                [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
                // left
                [1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0],
                // back
                [1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0],
                // bottom
                [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            )),
            indices: Some(vec!(
                0, 1, 2, 2, 3, 0,
                4, 5, 6, 6, 7, 4,
                8, 9, 10, 10, 11, 8,
                12, 13, 14, 14, 15, 12,
                16, 17, 18, 18, 19, 16,
                20, 21, 22, 22, 23, 20,
            )),
            ..Default::default()
        }
    }

    /// Loads vertices buffer
    pub fn load_vertices_buffer(&mut self, device: &wgpu::Device, buffer: &[u8]) {
        self.vertices_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Static Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(buffer),
            usage: wgpu::BufferUsage::VERTEX,
        }));
    }

    /// Loads indices buffer
    pub fn load_indices_buffer(&mut self, device: &wgpu::Device) {
        self.indices_buffer = self.indices
            .as_ref()
            .map(|indices| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsage::INDEX,
                })
            });
    }

    /// Loads the [`Mesh`] buffers for static [`crate::components::Model`]
    pub fn load_as_static(&mut self, device: &wgpu::Device) {
        if self.vertices_buffer.is_some() {
            return;
        }
        let vertices = self.as_static()
            .expect("Mesh is not suitable for a static model");
        self.vertices_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Static Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        }));
        self.load_indices_buffer(device);
    }

    /// Loads the [`Mesh`] buffers for [`crate::components::Model`] with [`crate::assets::Skin`]
    pub fn load_as_skinned(&mut self, device: &wgpu::Device) {
        if self.vertices_buffer.is_some() {
            return;
        }
        let vertices = self.as_skinned()
            .expect("Mesh is not suitable for a skinned model");
        self.vertices_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skinned Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        }));
        self.load_indices_buffer(device);
    }

    /// Calculates Mesh missing data (normals)
    pub fn calculate(&mut self) {
        if self.normals.is_none() {
            let mut normals = vec![[99.9; 3]; self.positions.len()];
            let faces = self.indices
                .as_ref()
                .map(|i| i.len())
                .unwrap_or_else(|| self.positions.len()) / 3;

            for face in 0..faces {
                let mut i0 = (face * 3) as usize;
                let mut i1 = i0 + 1;
                let mut i2 = i1 + 1;
                if let Some(indices) = self.indices.as_ref() {
                    i0 = indices[i0] as usize;
                    i1 = indices[i1] as usize;
                    i2 = indices[i2] as usize;
                }
                let v0 = Vec3::from(self.positions[i0]);
                let v1 = Vec3::from(self.positions[i1]);
                let v2 = Vec3::from(self.positions[i2]);
                let n = (v1 - v0).cross(v2 - v1).normalize();
                // println!("normal: {:?}, {:?}, {:?} -> {:?}", v0, v1, v2, n);
                normals[i0] = if normals[i0][0] > 9.0 { n.into() } else { n.lerp(normals[i0].into(), 0.5).into() };
                normals[i1] = if normals[i1][0] > 9.0 { n.into() } else { n.lerp(normals[i1].into(), 0.5).into() };
                normals[i2] = if normals[i2][0] > 9.0 { n.into() } else { n.lerp(normals[i2].into(), 0.5).into() };
            }
            self.normals = Some(normals);
        }
    }

    /// Unloads the [`Mesh`] buffers
    pub fn unload(&mut self) {
        self.vertices_buffer.take();
        self.indices_buffer.take();
    }
}

/// Abstraction for vertex data with attributes
pub trait VertexAttributes: Pod + Zeroable {
    /// Returns size of the vertex data
    fn size() -> wgpu::BufferAddress {
        std::mem::size_of::<Self>() as wgpu::BufferAddress
    }
}

/// Vertex data for static [`crate::components::Model`]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StaticModelVertex {
    /// Vertex coordinates
    pub position: [f32; 3],
    /// Normal vector at the position
    pub normal: [f32; 3],
    /// Texture coordinate for the vertex
    pub uv: [f32; 2],
}

unsafe impl Pod for StaticModelVertex {}
unsafe impl Zeroable for StaticModelVertex {}
impl VertexAttributes for StaticModelVertex {}

/// Vertex data for [`crate::components::Model`] with [`crate::assets::Skin`]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkinnedModelVertex {
    /// Vertex coordinates
    pub position: [f32; 3],
    /// Normal vector at the position
    pub normal: [f32; 3],
    /// Texture coordinate for the vertex
    pub uv: [f32; 2],
    /// Weights of the transformation for the vertex
    pub weights: [f32; 4],
    /// Joints affecting the vertex transformation
    pub joints: [u16; 4],
}

unsafe impl Pod for SkinnedModelVertex {}
unsafe impl Zeroable for SkinnedModelVertex {}
impl VertexAttributes for SkinnedModelVertex {}
