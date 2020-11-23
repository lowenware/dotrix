use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3], // TODO: switch to 3, and add 4th in shader
    pub normal: [f32; 3],
    pub texture: [f32; 2],
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], texture: [f32; 2]) -> Self {
        Vertex {
            position,
            normal,
            texture,
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
    pub joints: Option<Vec<[u16; 4]>>,
    pub weights: Option<Vec<[f32; 4]>>,
}

impl Mesh {
    pub fn new(
        positions: Vec<[f32; 3]>,
        normals: Option<Vec<[f32; 3]>>,
        texture: Option<Vec<[f32; 2]>>,
        indices: Option<Vec<u32>>,
        joints: Option<Vec<[u16; 4]>>,
        weights: Option<Vec<[f32; 4]>>,
    ) -> Self {
        let mut vertices: Vec<Vertex> = positions
            .iter()
            .map(|p| Vertex::new(*p, [0.0, 0.0, 0.0], [0.0, 0.0]))
            .collect();

        if let Some(normals) = normals {
            for (v, n) in vertices.iter_mut().zip(normals.iter()) {
                v.normal = *n;
            }
        }

        if let Some(texture) = texture {
            for (v, t) in vertices.iter_mut().zip(texture.iter()) {
                v.texture = *t;
            }
        }

        Self {
            vertices,
            indices,
            joints,
            weights,
        }
    }

    pub fn cube() -> Self {
        Self {
            vertices: vec!(
                // front
                Vertex::new([-1.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
                Vertex::new([1.0, 0.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
                Vertex::new([1.0, 2.0, 1.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
                Vertex::new([-1.0, 2.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
                // top 
                Vertex::new([1.0, 2.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0]),
                Vertex::new([-1.0, 2.0, -1.0], [0.0, 1.0, 0.0], [0.0, 0.0]),
                Vertex::new([-1.0, 2.0, 1.0], [0.0, 1.0, 0.0], [0.0, 1.0]),
                Vertex::new([1.0, 2.0, 1.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
                // right
                Vertex::new([1.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 0.0]),
                Vertex::new([1.0, 2.0, -1.0], [1.0, 0.0, 0.0], [1.0, 0.0]),
                Vertex::new([1.0, 2.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0]),
                Vertex::new([1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0]),
                // left
                Vertex::new([-1.0, 0.0, 1.0], [-1.0, 0.0, 0.0], [1.0, 0.0]),
                Vertex::new([-1.0, 2.0, 1.0], [-1.0, 0.0, 0.0], [0.0, 0.0]),
                Vertex::new([-1.0, 2.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0]),
                Vertex::new([-1.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [1.0, 1.0]),
                // back
                Vertex::new([-1.0, 2.0, -1.0], [0.0, 0.0, -1.0], [1.0, 0.0]),
                Vertex::new([1.0, 2.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0]),
                Vertex::new([1.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 1.0]),
                Vertex::new([-1.0, 0.0, -1.0], [0.0, 0.0, -1.0], [1.0, 1.0]),
                // bottom
                Vertex::new([1.0, 0.0, 1.0], [0.0, -1.0, 0.0], [0.0, 0.0]),
                Vertex::new([-1.0, 0.0, 1.0], [0.0, -1.0, 0.0], [1.0, 0.0]),
                Vertex::new([-1.0, 0.0, -1.0], [0.0, -1.0, 0.0], [1.0, 1.0]),
                Vertex::new([1.0, 0.0, -1.0], [0.0, -1.0, 0.0], [0.0, 1.0]),
            ),

            indices: Some(vec!(
                0, 1, 2, 2, 3, 0,
                4, 5, 6, 6, 7, 4,
                8, 9, 10, 10, 11, 8,
                12, 13, 14, 14, 15, 12,
                16, 17, 18, 18, 19, 16,
                20, 21, 22, 22, 23, 20,
            )),

            joints: None,
            weights: None,
        }
    }

    pub fn cube2() -> Self {
        Self {
            vertices: vec!(
                // front
                Vertex::new([-4.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
                Vertex::new([-2.0, 0.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
                Vertex::new([-2.0, 2.0, 1.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
                Vertex::new([-4.0, 2.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
                // top 
                Vertex::new([-2.0, 2.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0]),
                Vertex::new([-4.0, 2.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]),
                Vertex::new([-4.0, 2.0, 1.0], [0.0, 1.0, 0.0], [0.0, 1.0]),
                Vertex::new([-2.0, 2.0, 1.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
                // right
                Vertex::new([-2.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0]),
                Vertex::new([-2.0, 2.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0]),
                Vertex::new([-2.0, 2.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0]),
                Vertex::new([-2.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0]),
                // left
                Vertex::new([-4.0, 0.0, 1.0], [-1.0, 0.0, 0.0], [1.0, 0.0]),
                Vertex::new([-4.0, 2.0, 1.0], [-1.0, 0.0, 0.0], [0.0, 0.0]),
                Vertex::new([-4.0, 2.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0]),
                Vertex::new([-4.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 1.0]),
                // back
                Vertex::new([-4.0, 2.0, 0.0], [0.0, 0.0, -1.0], [1.0, 0.0]),
                Vertex::new([-2.0, 2.0, 0.0], [0.0, 0.0, -1.0], [0.0, 0.0]),
                Vertex::new([-2.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0]),
                Vertex::new([-4.0, 0.0, 0.0], [0.0, 0.0, -1.0], [1.0, 1.0]),
                // bottom
                Vertex::new([-2.0, 0.0, 1.0], [0.0, -1.0, 0.0], [0.0, 0.0]),
                Vertex::new([-4.0, 0.0, 1.0], [0.0, -1.0, 0.0], [1.0, 0.0]),
                Vertex::new([-4.0, 0.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0]),
                Vertex::new([-2.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 1.0]),
            ),

            indices: Some(vec!(
                0, 1, 2, 2, 3, 0,
                4, 5, 6, 6, 7, 4,
                8, 9, 10, 10, 11, 8,
                12, 13, 14, 14, 15, 12,
                16, 17, 18, 18, 19, 16,
                20, 21, 22, 22, 23, 20,
            )),

            joints: None,
            weights: None,
        }
    }
}
