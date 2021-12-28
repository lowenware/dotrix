//! Rendering service and system, pipelines, abstractions for models, transformation, skybox,
//! lights and overlay
mod backend;

use backend::Context as Backend;
use dotrix_math::Mat4;

use crate::assets::{Mesh, Shader};
use crate::ecs::{Const, Mut};
use crate::{Assets, Color, Globals, Pipeline, Window};

pub use backend::{
    Bindings, PipelineBackend, Sampler, ShaderModule, StorageBuffer, TextureBuffer, UniformBuffer,
    VertexBuffer,
};

/// Conversion matrix
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

const RENDERER_STARTUP: &str =
    "Please, use `renderer::startup` as a first system on the `startup` run level";

/// Scissors Rectangle
pub struct ScissorsRect {
    /// Minimal clip size by X axis
    pub clip_min_x: u32,
    /// Minimal clip size by Y axis
    pub clip_min_y: u32,
    /// widget width
    pub width: u32,
    /// widget height
    pub height: u32,
}

/// Pipeline options
#[derive(Default)]
pub struct Options {
    /// Scissors Rectangle
    pub scissors_rect: Option<ScissorsRect>,
}

/// Service providing an interface to `WGPU` and `WINIT`
pub struct Renderer {
    clear_color: Color,
    cycle: usize,
    backend: Option<Backend>,
    loaded: bool,
}

impl Renderer {
    fn backend(&self) -> &Backend {
        self.backend.as_ref().expect(RENDERER_STARTUP)
    }

    fn backend_mut(&mut self) -> &mut Backend {
        self.backend.as_mut().expect(RENDERER_STARTUP)
    }

    /// Returns the rendering cycle number (Experimental)
    pub fn cycle(&self) -> usize {
        self.cycle
    }

    /// Laods the vertex buffer to GPU
    pub fn load_vertex_buffer<'a>(
        &self,
        buffer: &mut VertexBuffer,
        attributes: &'a [u8],
        indices: Option<&'a [u8]>,
        count: usize,
    ) {
        buffer.load(self.backend(), attributes, indices, count as u32);
    }

    /// Loads the texture buffer to GPU
    pub fn load_texture_buffer<'a>(
        &self,
        buffer: &mut TextureBuffer,
        width: u32,
        height: u32,
        layers: &'a [&'a [u8]],
    ) {
        buffer.load(self.backend(), width, height, layers);
    }

    /// Loads the uniform buffer to GPU
    pub fn load_uniform_buffer<'a>(&self, buffer: &mut UniformBuffer, data: &'a [u8]) {
        buffer.load(self.backend(), data);
    }

    /// Loads the sampler to GPU
    pub fn load_sampler(&self, sampler: &mut Sampler) {
        sampler.load(self.backend());
    }

    /// Loads the storage buffer to GPU
    pub fn load_storage_buffer<'a>(&self, buffer: &mut StorageBuffer, data: &'a [u8]) {
        buffer.load(self.backend(), data);
    }

    /// Loads the sahder module to GPU
    pub fn load_shader_module(&self, shader_module: &mut ShaderModule, name: &str, code: &str) {
        shader_module.load(self.backend(), name, code);
    }

    /// Forces engine to reload shaders
    pub fn reload(&mut self) {
        self.loaded = false;
    }

    /// Binds uniforms and other data to the pipeline
    pub fn bind(&mut self, pipeline: &mut Pipeline, layout: PipelineLayout) {
        if !self.backend().has_pipeline(pipeline.shader) {
            let pipeline_backend = PipelineBackend::new(self.backend(), &layout);
            self.backend_mut()
                .add_pipeline(pipeline.shader, pipeline_backend);
        }

        let pipeline_backend = self.backend().pipeline(pipeline.shader).unwrap();

        let mut bindings = Bindings::default();
        bindings.load(self.backend(), pipeline_backend, layout.bindings);
        pipeline.bindings = bindings;
    }

    /// Runs the pipeline for a mesh
    pub fn run(&mut self, pipeline: &mut Pipeline, mesh: &Mesh) {
        // TODO: it is not good to copy backend here, find another solution
        // let mut backend = self.backend.take();
        self.backend_mut().run_pipeline(
            pipeline.shader,
            &mesh.vertex_buffer,
            &pipeline.bindings,
            &pipeline.options,
        );
        // self.backend = backend;
    }
}

impl Default for Renderer {
    /// Constructs new instance of the service
    fn default() -> Self {
        Renderer {
            clear_color: Color::from([0.1, 0.2, 0.3, 1.0]),
            cycle: 1,
            backend: None,
            loaded: false,
        }
    }
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

/// Startup system
pub fn startup(mut renderer: Mut<Renderer>, mut globals: Mut<Globals>, window: Mut<Window>) {
    // Init backend backend
    if renderer.backend.is_none() {
        renderer.backend = Some(futures::executor::block_on(backend::init(window.get())));
    }

    // Create texture sampler and store it with Globals
    let mut sampler = Sampler::default();
    renderer.load_sampler(&mut sampler);
    globals.set(sampler);
}

/// Frame binding system
pub fn bind(mut renderer: Mut<Renderer>, mut assets: Mut<Assets>) {
    let clear_color = renderer.clear_color;
    renderer.backend_mut().bind_frame(&clear_color);

    if renderer.loaded {
        return;
    }

    let mut loaded = true;

    for (_id, shader) in assets.iter_mut::<Shader>() {
        shader.load(&renderer);
        if !shader.loaded() {
            loaded = false;
        }
    }

    renderer.loaded = loaded;
}

/// Frame release system
pub fn release(mut renderer: Mut<Renderer>) {
    renderer.backend_mut().release_frame();
    renderer.cycle += 1;
    if renderer.cycle == 0 {
        renderer.cycle = 1;
    }
}

/// Resize handling system
pub fn resize(mut renderer: Mut<Renderer>, window: Const<Window>) {
    let size = window.inner_size();
    renderer.backend_mut().resize(size.x, size.y);
}

/// Pipeline options
pub struct PipelineOptions {
    /// Depth buffer mode
    pub depth_buffer_mode: DepthBufferMode,
    /// Disable cull mode
    pub disable_cull_mode: bool,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            depth_buffer_mode: DepthBufferMode::Write,
            disable_cull_mode: false,
        }
    }
}

/// Pipeline layout
pub struct PipelineLayout<'a> {
    /// Name of the Pipeline
    pub label: String,
    /// Mesh object to construct the pipeline
    pub mesh: &'a Mesh,
    /// Shader module
    pub shader: &'a Shader,
    /// Pipeline bindings
    pub bindings: &'a [BindGroup<'a>],
    /// Pipeline options
    pub options: PipelineOptions,
}

/// Mode of the depth buffer
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DepthBufferMode {
    /// Read Only mode
    Read,
    /// Read + Write mode
    Write,
    /// Depth buffer is disabled
    Disabled,
}

/// Vertex Attribute Format
#[derive(Debug)]
pub enum AttributeFormat {
    /// 32 bit float attribute
    Float32,
    /// 2 x 32 bit float attribute
    Float32x2,
    /// 3 x 32 bit float attribute
    Float32x3,
    /// 4 x 32 bit float attribute
    Float32x4,
    /// 2 x 16 bit unsigned integer attribute
    Uint16x2,
    /// 4 x 16 bit unsigned integer attribute
    Uint16x4,
    /// 32 bit unsigned integer attribute
    Uint32,
    /// 2 x 32 bit unsigned integer attribute
    Uint32x2,
    /// 3 x 32 bit unsigned integer attribute
    Uint32x3,
    /// 4 x 32 bit unsigned integer attribute
    Uint32x4,
}

impl AttributeFormat {
    /// Returns the actual attribute size in bytes
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

/// Binding types (Label, Stage, Buffer)
pub enum Binding<'a> {
    /// Uniform binding
    Uniform(&'a str, Stage, &'a UniformBuffer),
    /// Texture binding
    Texture(&'a str, Stage, &'a TextureBuffer),
    /// 3D Texture binding
    Texture3D(&'a str, Stage, &'a TextureBuffer),
    /// Texture sampler binding
    Sampler(&'a str, Stage, &'a Sampler),
    /// Texture sampler binding
    Storage(&'a str, Stage, &'a StorageBuffer),
}

/// Rendering stage
pub enum Stage {
    /// Vertex shader stage
    Vertex,
    /// Fragment shader stage
    Fragment,
    /// Compute shader stage
    Compute,
    /// Any stage
    All,
}

/// Bind Group holding bindings
pub struct BindGroup<'a> {
    label: &'a str,
    bindings: Vec<Binding<'a>>,
}

impl<'a> BindGroup<'a> {
    /// Constructs new Bind Group
    pub fn new(label: &'a str, bindings: Vec<Binding<'a>>) -> Self {
        Self { label, bindings }
    }
}
