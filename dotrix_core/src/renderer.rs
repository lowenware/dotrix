//! Rendering service and systems
mod access;
mod bindings;
mod buffer;
mod context;
mod mesh;
mod pipelines;
mod sampler;
mod shader;
mod texture;

use dotrix_math::Mat4;

use crate::assets::{Mesh, Shader};
use crate::ecs::{Const, Mut};
use crate::{Assets, Color, Globals, Id, Window};

pub use access::Access;
pub use bindings::{BindGroup, Binding, Bindings, Stage};
pub use buffer::Buffer;
pub use context::Context;
pub use mesh::AttributeFormat;
pub use pipelines::{
    Compute, ComputeArgs, ComputeOptions, DepthBufferMode, DrawArgs, Pipeline, PipelineInstance,
    PipelineLayout, Render, RenderOptions, ScissorsRect, WorkGroups,
};
pub use sampler::Sampler;
pub use shader::ShaderModule;
pub use texture::Texture;

// Ree-export native wgpu module
pub use wgpu;

/// Conversion matrix
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

const RENDERER_STARTUP: &str =
    "Please, use `renderer::startup` as a first system on the `startup` run level";

/// Service providing an interface to `WGPU` and `WINIT`
pub struct Renderer {
    /// Surface clear color
    pub clear_color: Color,
    /// Auto-incrementing rendering cylce
    pub cycle: usize,
    /// Antialiasing
    pub antialiasing: Antialiasing,
    /// Low-level rendering context
    pub context: Option<Context>,
    /// When dirty, renderer will try to load missing pipelines on frame binding
    pub dirty: bool,
}

impl Renderer {
    /// Sets default clear color
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    fn context(&self) -> &Context {
        self.context.as_ref().expect(RENDERER_STARTUP)
    }

    fn context_mut(&mut self) -> &mut Context {
        self.context.as_mut().expect(RENDERER_STARTUP)
    }

    /// Returns the rendering cycle number (Experimental)
    pub fn cycle(&self) -> usize {
        self.cycle
    }

    /// Laods the vertex buffer to GPU
    /*
    pub fn load_mesh<'a>(
        &self,
        buffer: &mut VertexBuffer,
        attributes: &'a [u8],
        indices: Option<&'a [u8]>,
        count: usize,
    ) {
        buffer.load(self.context(), attributes, indices, count as u32);
    }*/

    /// Loads the texture buffer to GPU
    pub fn load_texture<'a>(
        &self,
        texture: &mut Texture,
        width: u32,
        height: u32,
        layers: &'a [&'a [u8]],
    ) {
        texture.load(self.context(), width, height, layers);
    }

    /// Loads the buffer to GPU
    pub fn load_buffer<'a>(&self, buffer: &mut Buffer, data: &'a [u8]) {
        buffer.load(self.context(), data);
    }

    /// Create a buffer on GPU without data
    pub fn create_buffer(&self, buffer: &mut Buffer, size: u32) {
        buffer.create(self.context(), size);
    }

    /// Loads the sampler to GPU
    pub fn load_sampler(&self, sampler: &mut Sampler) {
        sampler.load(self.context());
    }

    /// Loads the sahder module to GPU
    pub fn load_shader(&self, shader_module: &mut ShaderModule, code: &str) {
        shader_module.load(self.context(), code);
    }

    /// Forces engine to reload shaders
    pub fn reload(&mut self) {
        self.dirty = true;
        self.drop_all_pipelines();
    }

    /// Drop the context pipeline for a shader
    ///
    /// This should be called when a shader is removed.
    pub fn drop_pipeline(&mut self, shader: Id<Shader>) {
        self.dirty = true;
        self.context_mut().drop_pipeline(shader);
    }

    /// Returns true if renderer has pipeline for the sahder
    pub fn has_pipeline(&self, shader: Id<Shader>) -> bool {
        self.context().has_pipeline(shader)
    }

    /// Drop all loaded context pipelines for all shader
    pub fn drop_all_pipelines(&mut self) {
        self.dirty = true;
        self.context_mut().drop_all_pipelines();
    }

    /// Binds uniforms and other data to the pipeline
    pub fn bind(&mut self, pipeline: &mut Pipeline, layout: PipelineLayout) {
        if !self.context().has_pipeline(pipeline.shader) {
            let instance = layout.instance(self.context());
            self.context_mut().add_pipeline(pipeline.shader, instance);
        }

        let instance = self.context().pipeline(pipeline.shader).unwrap();
        let mut bindings = Bindings::default();
        let bindings_layout = match layout {
            PipelineLayout::Render { bindings, .. } => bindings,
            PipelineLayout::Compute { bindings, .. } => bindings,
        };
        bindings.load(self.context(), instance, bindings_layout);
        pipeline.bindings = bindings;
    }

    /// Runs the render pipeline for a mesh
    pub fn draw(&mut self, pipeline: &mut Pipeline, mesh: &Mesh, args: &DrawArgs) {
        self.context_mut()
            .run_render_pipeline(pipeline.shader, mesh, &pipeline.bindings, args);
    }

    /// Runs the compute pipeline
    pub fn compute(&mut self, pipeline: &mut Pipeline, args: &ComputeArgs) {
        self.context_mut()
            .run_compute_pipeline(pipeline.shader, &pipeline.bindings, args);
    }

    /// Copy a texture to a buffer
    pub fn copy_texture_to_buffer(
        &mut self,
        texture: &Texture,
        buffer: &Buffer,
        extent: [u32; 3],
        bytes_per_pixel: u32,
    ) {
        self.context_mut()
            .run_copy_texture_to_buffer(texture, buffer, extent, bytes_per_pixel);
    }
}

/// Antialiasing modes enumeration
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Antialiasing {
    /// Enable antialiasing
    Enabled,
    /// Disable antialiasing
    Disabled,
    /// Manual control of number of samples for multisampled antialiasing
    MSAA {
        /// Number od samples for MSAA
        sample_count: u32,
    },
}

impl Antialiasing {
    /// get sample count for the antaliasing mode
    pub fn sample_count(self) -> u32 {
        match self {
            Antialiasing::Enabled => 4,
            Antialiasing::Disabled => 1,
            Antialiasing::MSAA { sample_count } => sample_count,
        }
    }
}

impl Default for Renderer {
    /// Constructs new instance of the service
    fn default() -> Self {
        Renderer {
            clear_color: Color::from([0.1, 0.2, 0.3, 1.0]),
            cycle: 1,
            context: None,
            dirty: true,
            antialiasing: Antialiasing::Enabled,
        }
    }
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

/// Startup system
pub fn startup(mut renderer: Mut<Renderer>, mut globals: Mut<Globals>, window: Mut<Window>) {
    // get sample count
    let sample_count = renderer.antialiasing.sample_count();
    // Init context
    if renderer.context.is_none() {
        renderer.context = Some(futures::executor::block_on(context::init(
            window.get(),
            sample_count,
        )));
    }

    // Create texture sampler and store it with Globals
    let mut sampler = Sampler::default();
    renderer.load_sampler(&mut sampler);
    globals.set(sampler);
}

/// Frame binding system
pub fn bind(mut renderer: Mut<Renderer>, mut assets: Mut<Assets>) {
    let clear_color = renderer.clear_color;
    let sample_count = renderer.antialiasing.sample_count();
    // NOTE: other option here is to check sample_count != context.sample_count
    let reload_request = renderer
        .context_mut()
        .bind_frame(&clear_color, sample_count);

    if !renderer.dirty && !reload_request {
        return;
    }

    let mut loaded = true;

    for (_id, shader) in assets.iter_mut::<Shader>() {
        shader.load(&renderer);
        if !shader.loaded() {
            loaded = false;
        }
    }

    renderer.dirty = !loaded;
}

/// Frame release system
pub fn release(mut renderer: Mut<Renderer>) {
    renderer.context_mut().release_frame();
    renderer.cycle += 1;
    if renderer.cycle == 0 {
        renderer.cycle = 1;
    }
    // Check for resource cleanups and mapping callbacks
    if let Some(context) = renderer.context.as_ref() {
        context.device.poll(wgpu::Maintain::Poll);
    }
}

/// Resize handling system
pub fn resize(mut renderer: Mut<Renderer>, window: Const<Window>) {
    let size = window.inner_size();
    renderer.context_mut().resize(size.x, size.y);
}
