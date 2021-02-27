use bytemuck::{ Pod, Zeroable };
use wgpu::util::DeviceExt;
use super::mesh::VertexAttributes;

/// Asset for building [`crate::components::WireFrame`]
///
/// It is similar to [`super::Mesh`], but with one major difference. Instead of polygons (Triangle
/// topology) it implements a storage for lines.
#[derive(Default)]
pub struct Wires {
    /// Vertices of lines beginings and ends
    pub positions: Vec<[f32; 3]>,
    /// Colors of vertices
    pub colors: Vec<[f32; 3]>,
    /// Pipeline buffers
    pub vertices_buffer: Option<wgpu::Buffer>,
}

impl Wires {

    /// Constructs [`Wires`] that can be rendered as wired cube
    pub fn cube(color: [f32; 3]) -> Self {
        Self {
            positions: vec!(
                // front
                [-1.0, -1.0, 1.0], [1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0], [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0], [-1.0, -1.0, 1.0],
                // top 
                [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0],
                [-1.0, 1.0, -1.0], [-1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0], [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0], [1.0, 1.0, -1.0],
                // right
                [1.0, -1.0, -1.0], [1.0, 1.0, -1.0],
                [1.0, 1.0, -1.0], [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0], [1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0], [1.0, -1.0, -1.0],
                // left
                [-1.0, -1.0, 1.0], [-1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0], [-1.0, 1.0, -1.0],
                [-1.0, 1.0, -1.0], [-1.0, -1.0, -1.0],
                [-1.0, -1.0, -1.0], [-1.0, -1.0, 1.0],
                // back
                [-1.0, 1.0, -1.0], [1.0, 1.0, -1.0],
                [1.0, 1.0, -1.0], [1.0, -1.0, -1.0],
                [1.0, -1.0, -1.0], [-1.0, -1.0, -1.0],
                [-1.0, -1.0, -1.0], [-1.0, 1.0, -1.0],
                // bottom
                [1.0, -1.0, 1.0], [-1.0, -1.0, 1.0],
                [-1.0, -1.0, 1.0], [-1.0, -1.0, -1.0],
                [-1.0, -1.0, -1.0], [1.0, -1.0, -1.0],
                [1.0, -1.0, -1.0], [1.0, -1.0, 1.0],
            ),
            colors: vec!(color; 48),
            ..Default::default()
        }
    }

    /// Returns number of vertices in the [`Wires`]
    pub fn vertices_count(&self) -> u32 {
        self.positions.len() as u32
    }

    /// Returns vertices of the [`Wires`] packed for shaders
    pub fn vertices(&self) -> Vec<WireFrameVertex> {
        self.positions
            .iter()
            .zip(self.colors.iter())
            .map(|(&position, &color)| {
                WireFrameVertex {
                    position,
                    color,
                }
            })
            .collect::<Vec<_>>()
    }

    /// Loads buffers
    pub fn load(&mut self, device: &wgpu::Device) {
        if self.vertices_buffer.is_some() {
            return;
        }
        let vertices = self.vertices();
        self.vertices_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wires Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        }));
    }

    /// Unloads buffers
    pub fn unload(&mut self) {
        self.vertices_buffer.take();
    }
}

/// Packed for shaders [`Wires`] vertex
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WireFrameVertex {
    /// Vertex position
    pub position: [f32; 3],
    /// Vertex color
    pub color: [f32; 3],
}

unsafe impl Pod for WireFrameVertex {}
unsafe impl Zeroable for WireFrameVertex {}
impl VertexAttributes for WireFrameVertex {}

