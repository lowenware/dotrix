use wgpu;


/// Rendering Pipeline
pub struct Pipeline {
    /// WGPU bind group layout
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// WGPU pipeline
    pub wgpu_pipeline: wgpu::RenderPipeline,
}

pub struct RenderPipeline {
    /// Shader asset ID
    pub shader: Id<Shader>,
    /// Vertex bufffer layout
    pub vertex_buffer_layout: Vec<VertexAttribute>,
    /// Pipeline bindings layout
    pub bindings_layout: Vec<Bind>,
    /// Depth buffer option
    pub use_depth_buffer: bool
}


impl RenderPipelineRunner {
}
