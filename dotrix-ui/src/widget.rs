use dotrix_gpu as gpu;
use dotrix_log as log;
use dotrix_mesh::Mesh;
use dotrix_types::vertex;
use dotrix_types::{Color, Id};

use crate::Rect;

pub type VertexAttributes = (Position, vertex::TexUV, Color<u8>);

pub struct Position {
    pub value: [f32; 2],
}

impl vertex::Attribute for Position {
    type Raw = [f32; 2];
    fn name() -> &'static str {
        "Screen Position"
    }
    fn format() -> vertex::AttributeFormat {
        vertex::AttributeFormat::Float32x2
    }
}

pub struct Widget {
    pub rect: Rect,
    pub mesh: Mesh,
    pub texture: Id<gpu::TextureView>,
}

impl Widget {
    pub fn new(rect: Rect, mesh: Mesh, texture: Id<gpu::TextureView>) -> Self {
        if !mesh.contains::<VertexAttributes>() {
            panic!("Widget mesh must contain Position, TexUV and Color<u8>");
        }
        Self {
            rect,
            mesh,
            texture,
        }
    }
}

pub struct Builder {
    pub positions: Vec<[f32; 2]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<u32>,
    pub indices: Vec<u32>,
    pub rect: Rect,
    pub texture: Id<gpu::TextureView>,
}

impl Builder {
    pub fn new(rect: Rect, texture: Id<gpu::TextureView>) -> Self {
        Self {
            positions: vec![],
            uvs: vec![],
            colors: vec![],
            indices: vec![],
            rect,
            texture,
        }
    }
}

impl From<Builder> for Widget {
    fn from(builder: Builder) -> Self {
        let mut mesh = Mesh::new(String::from("widget"));
        mesh.set_vertices::<Position>(builder.positions);
        mesh.set_vertices::<vertex::TexUV>(builder.uvs);
        mesh.set_vertices::<Color<u8>>(builder.colors);
        mesh.set_indices(builder.indices);

        log::debug!("Vertices: {:?}", mesh.vertices::<Position>());
        log::debug!("Indices: {:?}", mesh.indices::<u32>());

        Self {
            rect: builder.rect,
            mesh,
            texture: builder.texture,
        }
    }
}
