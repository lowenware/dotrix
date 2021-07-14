use wgpu;
use crate::assets::{ Id, Shader };
use super::{ PipelineBackend as RenderBackend, Sampler, TextureBuffer, UniformBuffer, VertexBuffer };

/// Rendering Pipeline
pub enum Pipeline {
    Render(Render),
}


#[derive(Default)]
pub struct Render {
    pub label: String,
    pub shader: Id<Shader>,
    /// Vertex bufffer layout
    pub vertices: Vec<AttributeFormat>,
    /// Pipeline bindings layout
    pub globals: Vec<Bind>,
    pub locals: Vec<Bind>,
    pub extras: Vec<Bind>,
    /// Depth buffer option
    pub use_depth_buffer: bool,

    pub backend: Option<RenderBackend>,
}

pub mod Stage {
    pub struct Vertex(pub bool);
    pub struct Fragment(pub bool);

    impl Vertex {
        pub fn visible(&self) -> bool {
            return self.0;
        }
    }

    impl Fragment {
        pub fn visible(&self) -> bool {
            return self.0;
        }
    }
}

/// Binding types
pub enum Bind {
    /// Uniform binding
    Uniform(&'static str, Stage::Vertex, Stage::Fragment),
    /// 2D Texture binding
    Texture(&'static str, Stage::Vertex, Stage::Fragment),
    // 3D Texture binding
    // Texture3d(To),
    /// Texture sampler binding
    Sampler(&'static str, Stage::Vertex, Stage::Fragment),
}

pub enum AttributeFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint16x2,
    Uint16x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
}

impl AttributeFormat {
    pub fn size(&self) -> usize {
        match self {
            AttributeFormat::Float32 => 4,
            AttributeFormat::Float32x2 => 4 * 2,
            AttributeFormat::Float32x3 => 4 * 3,
            AttributeFormat::Float32x4 => 4 * 4,
            AttributeFormat::Uint16x2 => 2 * 2,
            AttributeFormat::Uint16x4 => 2 * 4,
            AttributeFormat::Uint32 => 4,
            AttributeFormat::Uint32x2 => 4 * 2,
            AttributeFormat::Uint32x3 => 4 * 3,
            AttributeFormat::Uint32x4 => 4 * 4,
        }
    }
}



/// Binding type enumerator
pub enum Binding<'a> {
    Texture(&'a TextureBuffer),
    Uniform(&'a UniformBuffer),
    Sampler(&'a Sampler),
}

/// Pipeline inputs
pub struct Bindings<'a, 'b> {
    pub globals: &'a [&'b Binding<'b>],
    pub locals: &'a [&'b Binding<'b>],
    pub extras: &'a [&'b Binding<'b>],
}

/*

/// Render pipeline
pub struct Pipeline {
    /// Shader asset ID
    pub shader: Id<Shader>,
    /// Vertex bufffer layout
    pub vertex_buffer_layout: Vec<VertexAttribute>,
    /// Pipeline bindings layout
    pub bindings_layout: Vec<Bind>,
    /// Depth buffer option
    pub use_depth_buffer: bool
}

impl Default for Pipeline {
    fn default() -> Self {
        Self {
            shader: Id::default(),
            vertex_buffer_layout: Vec::new(),
            bindings_layout: Vec::new(),
            use_depth_buffer: true,
        }
    }
}

/// Vertex Attribute Layout 
pub struct VertexAttribute {
    /// Vertex attribute size
    pub size: usize,
}

pub trait Renderable {
    fn vertex_buffers(&self) -> ?;
    fn bindings(&self) -> ?;
}
*/
