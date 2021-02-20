use bytemuck::{ Pod, Zeroable };
use wgpu::util::DeviceExt;
use super::mesh::VertexAttributes;

#[derive(Default)]
pub struct Wires {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub vertices_buffer: Option<wgpu::Buffer>,
}

impl Wires {

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

    pub fn vertices_count(&self) -> u32 {
        self.positions.len() as u32
    }

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

    pub fn unload(&mut self) {
        self.vertices_buffer.take();
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WireFrameVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

unsafe impl Pod for WireFrameVertex {}
unsafe impl Zeroable for WireFrameVertex {}
impl VertexAttributes for WireFrameVertex {}

