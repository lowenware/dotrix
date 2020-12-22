use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
    pub weights: Option<Vec<[f32; 4]>>,
    pub joints: Option<Vec<[u16; 4]>>,
    pub indices: Option<Vec<u32>>,
    pub vertices_buffer: Option<wgpu::Buffer>,
    pub indices_buffer: Option<wgpu::Buffer>,
}

impl Mesh {

    /// converts a mesh into a vector of vertices data, packed for static shadering:
    /// positions, normals, uvs
    pub fn as_static(&self) -> Result<Vec<StaticModelVertex>, ()> {
        if let Some(normals) = self.normals.as_ref() {
            if let Some(uvs) = self.uvs.as_ref() {
                return Ok(
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
        Err(())
    }

    /// converts a mesh into a vector of vertices data, packed for skinned shadering:
    /// positions, normals, uvs, weights, 
    pub fn as_skinned(&self) -> Result<Vec<SkinnedModelVertex>, ()> {
        if let Some(normals) = self.normals.as_ref() {
            if let Some(uvs) = self.uvs.as_ref() {
                if let Some(all_weights) = self.weights.as_ref() {
                    if let Some(all_joints) = self.joints.as_ref() {
                        let weights_joints = all_weights.iter().zip(all_joints.iter());
                        return Ok(
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
        Err(())
    }

    pub fn is_skinned(&self) -> bool {
        self.weights.is_some() && self.joints.is_some()
    }

    pub fn indices_count(&self) -> u32 {
        self.indices
            .as_ref()
            .map(|i| i.len())
            .unwrap_or_else(|| self.positions.len()) as u32
    }

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

    pub fn load_vertices_buffer(&mut self, device: &wgpu::Device, buffer: &[u8]) {
        self.vertices_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Static Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(buffer),
            usage: wgpu::BufferUsage::VERTEX,
        }));
    }

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

    pub fn load_as_static(&mut self, device: &wgpu::Device) {
        let vertices = self.as_static()
            .expect("Mesh is not suitable for a static model");
        self.vertices_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Static Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        }));
        self.load_indices_buffer(device);
    }

    pub fn load_as_skinned(&mut self, device: &wgpu::Device) {
        let vertices = self.as_skinned()
            .expect("Mesh is not suitable for a skinned model");
        self.vertices_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skinned Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        }));
        self.load_indices_buffer(device);
    }

    pub fn calculate(&mut self) {
        use cgmath::InnerSpace;
        if self.normals.is_none() {
            let mut normals = vec![[0.0; 3]; self.positions.len()];
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
                let v0 = cgmath::Vector3::from(self.positions[i0]);
                let v1 = cgmath::Vector3::from(self.positions[i1]);
                let v2 = cgmath::Vector3::from(self.positions[i2]);
                let n = (v1 - v0).cross(v2 - v1).normalize();
                normals[i0] = n.into();
                normals[i1] = n.into();
                normals[i2] = n.into();
            }
            self.normals = Some(normals);
        }
    }
}

pub trait VertexAttributes: Pod + Zeroable {
    fn size() -> wgpu::BufferAddress {
        std::mem::size_of::<Self>() as wgpu::BufferAddress
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StaticModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

unsafe impl Pod for StaticModelVertex {}
unsafe impl Zeroable for StaticModelVertex {}
impl VertexAttributes for StaticModelVertex {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkinnedModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub weights: [f32; 4],
    pub joints: [u16; 4],
}

unsafe impl Pod for SkinnedModelVertex {}
unsafe impl Zeroable for SkinnedModelVertex {}
impl VertexAttributes for SkinnedModelVertex {}
