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

use dotrix_math::{Mat4, Vec2};

use crate::assets::{Asset, Shader};
use crate::ecs::{Const, Mut};
use crate::providers::{BufferProvider, MeshProvider, TextureProvider};
use crate::reloadable::Reloadable;
use crate::{Assets, Color, Globals, Window};
use std::time::Instant;

pub use access::Access;
pub use bindings::{BindGroup, Binding, Bindings, Stage};
pub use buffer::Buffer;
pub use context::Context;
pub use mesh::AttributeFormat;
pub use pipelines::{
    Compute, ComputeArgs, ComputeOptions, ComputePipeline, DepthBufferMode, DrawArgs, Pipeline,
    PipelineInstance, PipelineLayout, Render, RenderOptions, RenderPipeline, ScissorsRect,
    WorkGroups,
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

/// Collection of traits that a gpu buffer needs
pub trait GpuBuffer: Reloadable + BufferProvider + Asset {}
impl<T: Reloadable + BufferProvider + Asset> GpuBuffer for T {}

/// Collection of traits that a gpu texture needs
pub trait GpuTexture: Reloadable + TextureProvider + Asset {}
impl<T: Reloadable + TextureProvider + Asset> GpuTexture for T {}

/// Collection of traits that a gpu mesh needs
pub trait GpuMesh: Reloadable + MeshProvider + Asset {}
impl<T: Reloadable + MeshProvider + Asset> GpuMesh for T {}

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
    /// When time when last reload occured.
    /// Used to track if a pipeline instance should reload
    pub last_reload: Instant,
    /// Time at which it was last flagged as dirty
    pub dirty: Instant,
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

    /// Loads the texture buffer to GPU.
    /// This will recreate the texture, as a result it must be rebound on any pipelines for changes
    /// to take effect
    pub fn load_texture<'a>(
        &self,
        texture: &mut Texture,
        width: u32,
        height: u32,
        layers: &'a [&'a [u8]],
    ) {
        texture.load(self.context(), width, height, layers);
    }

    /// Load data from cpu to a texture buffer on GPU
    /// This is a noop if texture has not been loaded with `load_texture`
    /// Unexpected results/errors occur if the dimensions differs from it dimensions at load time
    pub fn update_texture<'a>(
        &self,
        texture: &mut Texture,
        width: u32,
        height: u32,
        layers: &'a [&'a [u8]],
    ) {
        texture.update(self.context(), width, height, layers);
    }

    /// This will `[update_texture]` if texture has been loaded or `[load_texture]` if not
    /// the same cavets of `[update_texture]` apply in that care must be taken not to change
    /// the dimensions between `load` and `update`
    pub fn update_or_load_texture<'a>(
        &self,
        texture: &mut Texture,
        width: u32,
        height: u32,
        layers: &'a [&'a [u8]],
    ) {
        texture.update_or_load(self.context(), width, height, layers);
    }

    /// Loads the buffer to GPU
    pub fn load_buffer<'a>(&self, buffer: &mut Buffer, data: &'a [u8]) {
        buffer.load(self.context(), data);
    }

    /// Create a buffer on GPU without data
    pub fn create_buffer(&self, buffer: &mut Buffer, size: u32, mapped: bool) {
        buffer.create(self.context(), size, mapped);
    }

    /// Loads the sampler to GPU
    pub fn load_sampler(&self, sampler: &mut Sampler) {
        sampler.load(self.context());
    }

    /// Loads the sahder module to GPU
    pub fn load_shader(&self, shader_module: &mut ShaderModule, code: &str) {
        shader_module.load(self.context(), code);
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

    /// Fetch texture from GPU
    pub fn fetch_texture(
        &mut self,
        texture: &Texture,
        dimensions: [u32; 3],
    ) -> impl std::future::Future<Output = Result<Vec<u8>, wgpu::BufferAsyncError>> {
        texture.fetch_from_gpu(dimensions, self.context_mut())
    }

    /// Forces engine to reload shaders
    pub fn reload(&mut self) {
        self.dirty = Instant::now();
    }

    /// Binds uniforms and other data to the pipeline
    pub fn bind<'a, Mesh>(&mut self, pipeline: &mut Pipeline, layout: PipelineLayout<'a, Mesh>)
    where
        Mesh: GpuMesh,
        &'static Mesh: GpuMesh,
    {
        if !pipeline.bind_required(&layout) {
            return;
        }
        // Reload if pipeline is none or if the last reload is before the last dirty flag
        if pipeline.reload_required(self) {
            let instance = layout.instance(self.context());
            pipeline.instance = Some(instance);
        }

        let instance = pipeline.instance.as_ref().unwrap();
        let mut bindings = Bindings::default();
        let bindings_layout = match layout {
            PipelineLayout::Render { bindings, .. } => bindings,
            PipelineLayout::Compute { bindings, .. } => bindings,
        };
        bindings.load(self.context(), instance, bindings_layout);
        pipeline.bindings = bindings;
        pipeline.last_bound_at = Instant::now();
    }

    /// Runs the render pipeline for a mesh
    pub fn draw<Mesh: GpuMesh>(&mut self, pipeline: &mut Pipeline, mesh: &Mesh, args: &DrawArgs) {
        self.context_mut().run_render_pipeline(pipeline, mesh, args);
    }

    /// Runs the compute pipeline
    pub fn compute(&mut self, pipeline: &mut Pipeline, args: &ComputeArgs) {
        self.context_mut().run_compute_pipeline(pipeline, args);
    }

    /// Returns surface size
    pub fn surface_size(&self) -> Vec2 {
        let ctx = self.context();
        Vec2::new(ctx.sur_desc.width as f32, ctx.sur_desc.height as f32)
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
            dirty: Instant::now(),
            last_reload: Instant::now(),
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

    if renderer.dirty < renderer.last_reload && !reload_request {
        return;
    }

    let mut loaded = true;

    for (_id, shader) in assets.iter_mut::<Shader>() {
        shader.load(&renderer);
        if !shader.loaded() {
            loaded = false;
        }
    }

    if loaded {
        renderer.last_reload = Instant::now();
    }
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
