//! Rendering service and system, pipelines, abstractions for models, transformation, skybox,
//! lights and overlay
mod backend;
// pub mod bind_group_layout;
// pub mod color;
// mod light;
// pub mod pipeline;
// pub mod skybox;
// mod overlay;
// mod widget;
// mod wireframe;
// pub mod transform;


use std::collections::HashMap;

use dotrix_math::{ Mat4, Rad, perspective };
use backend::Context as Backend;

use crate::{
    assets::{ Mesh, Shader },
    components::Pipeline,
    ecs::{ Const, Mut, Context },
    generics::{ Id, Color },
    services::{ Assets, Camera, Globals, World },
    window::Window,
};

pub use backend::{
    Bindings,
    PipelineBackend,
    Sampler,
    ShaderModule,
    TextureBuffer,
    UniformBuffer,
    VertexBuffer,
};

/// Conversion matrix
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub const RENDERER_STARTUP: &str =
    "Please, use `renderer::startup` as a first system on the `startup` run level";

/*
struct PipelineEntry {
    pipeline: Pipeline,
    backend: Option<PipelineBackend>,
}
*/

/// Service providing an interface to `WGPU` and `WINIT`
pub struct Renderer {
    clear_color: Color,
    cycle: usize,
    backend: Option<Backend>,
    pipelines: HashMap<Id<Shader>, PipelineBackend>,
    loaded: bool,
    last_pipeline_id: u64,
    // Overlay providers storageOverlay providers storage
    // pub overlay: Vec<Overlay>,
}

impl Renderer {
    fn backend(&self) -> &Backend {
        self.backend.as_ref().expect(RENDERER_STARTUP)
    }

    fn backend_mut(&mut self) -> &mut Backend {
        self.backend.as_mut().expect(RENDERER_STARTUP)
    }

    pub fn cycle(&self) -> usize {
        self.cycle
    }

    // Adds an [`OverlayProvider`] to the service
    // pub fn add_overlay(&mut self, overlay_provider: Box<dyn OverlayProvider>) {
    //    self.overlay.push(Overlay::new(overlay_provider));
    // }

    // Returns the [`OverlayProvider`] previously added to the service, by it's type
    // pub fn overlay_provider<T: 'static + Send + Sync>(&self) -> Option<&T> {
    //    for overlay in &self.overlay {
    //        let provider = overlay.provider::<T>();
    //        if provider.is_some() {
    //            return provider;
    //        }
    //    }
    //    None
    // }

    /*
    /// Adds rendering [`Pipeline`] to the service and returns [`Id`] of it
    pub fn add_pipeline(&mut self, pipeline: Pipeline) -> Id<Pipeline> {
        let id = Id::new(self.pipelines.len() as u64 + 1);
        self.pipelines.insert(id, pipeline);
        id
    }

    /// Returns reference to a [`Pipeline`] by its [`Id`]
    pub fn pipeline(&self, id: Id<Pipeline>) -> &Pipeline {
        self.pipelines.get(&id)
            .expect("Pipeline has to be registered with `add_pipeline` method")
    }

    pub fn bind_pipeline(&self, id: Id<Pipeline>, bindings: &Bindings) -> Option<BindGroup> {
        if let Some(pipeline) = self.pipelines.get(&id) {
            if let Pipeline::Render(render_pipeline) = pipeline {
                if let Some(render_backend) = render_pipeline.backend.as_ref() {
                    let mut bind_group = BindGroup::default();
                    bind_group.load(&self.device, render_backend, bindings);
                    return Some(bind_group);
                }
            }
        }
        None
    }

    pub fn run_pipeline(&self, id: Id<Pipeline>, vertex_buffer: &VertexBuffer, bind_group: &BindGroup) {

    }
    */
    /*
    /// Adds a skybox [`Pipeline`] to the service and returns [`Id`] of it
    pub fn add_skybox_pipeline(&mut self) -> Id<Pipeline> {
        let pipeline = Pipeline::default_for_skybox(&self.device, &self.sc_desc);
        self.add_pipeline(pipeline)
    }

    /// Adds a non animated model [`Pipeline`] to the service and returns [`Id`] of it
    pub fn add_static_model_pipeline(&mut self) -> Id<Pipeline> {
        let pipeline = Pipeline::default_for_static_model(&self.device, &self.sc_desc);
        self.add_pipeline(pipeline)
    }

    /// Adds a skinned model [`Pipeline`] to the service and returns [`Id`] of it
    pub fn add_skinned_model_pipeline(&mut self) -> Id<Pipeline> {
        let pipeline = Pipeline::default_for_skinned_model(&self.device, &self.sc_desc);
        self.add_pipeline(pipeline)
    }

    /// Adds an overlay [`Pipeline`] to the service and returns [`Id`] of it
    pub fn add_overlay_pipeline(&mut self) -> Id<Pipeline> {
        let pipeline = Pipeline::default_for_overlay(&self.device, &self.sc_desc);
        self.add_pipeline(pipeline)
    }

    /// Adds a wire frame [`Pipeline`] to the service and returns [`Id`] of it
    pub fn add_wire_frame_pipeline(&mut self) -> Id<Pipeline> {
        let pipeline = Pipeline::default_for_wire_frame(
            &self.adapter,
            &self.device,
            &self.sc_desc
        );
        self.add_pipeline(pipeline)
    }
    */

    // Handler of the window resize event
    // pub fn resize(&mut self, width: u32, height: u32) {
    //    if width > 0 && height > 0 {
    //        self.sc_desc.width = width;
    //        self.sc_desc.height = height;

    //        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    //        self.depth_buffer = Self::create_depth_buffer(&self.device, width, height);

    //        let aspect_ratio = width as f32 / height as f32;
    //        let projection = Self::frustum(aspect_ratio);
    //        self.projection = projection;
    //    }
    // }

    /*
    /// Returns current swap chain frame instance
    pub fn frame(&self) -> Option<&backend::SwapChainFrame> {
        self.frame.as_ref()
    }

    /// Triggers swap chain of the frame
    pub fn next_frame(&mut self) {
        let frame = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
                self.swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture!")
            }
        };
        self.frame = Some(frame);
    }

    /// Finalizes frame rendering
    pub fn finalize(&mut self) {
        self.frame.take();
    }

    /// Returns a tuple of physical display size (width, height)
    pub fn display_size(&self, window: &Window) -> (u32, u32) {
        let size = window.inner_size();
        ( size.x, size.y )
    }

    /// Returns a tuple of virtual (scaled) display size (width, height)
    pub fn display_virtual_size(&self, window: &Window) -> (f32, f32) {
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        ( size.x as f32 / scale_factor, size.y as f32 / scale_factor )
    }

    /// Returns current scale factor of the display
    pub fn scale_factor(&self, window: &Window) -> f32 {
        window.scale_factor() as f32
    }

    */

    pub fn load_vertex_buffer<'a>(
        &self,
        buffer: &mut VertexBuffer,
        attributes: &'a [u8],
        indices: Option<&'a [u8]>,
        count: usize,
    ) {
        buffer.load(self.backend(), attributes, indices, count as u32);
    }

    pub fn load_texture_buffer<'a>(
        &self,
        buffer: &mut TextureBuffer,
        width: u32,
        height: u32,
        data: &'a [u8],
    ) {
        buffer.load(self.backend(), width, height, data);
    }

    pub fn load_uniform_buffer<'a>(&self, buffer: &mut UniformBuffer, data: &'a [u8]) {
        buffer.load(self.backend(), data);
    }

    pub fn load_sampler(&self, sampler: &mut Sampler) {
        sampler.load(self.backend());
    }

    pub fn load_shader_module(
        &self,
        shader_module: &mut ShaderModule,
        name: &str,
        code: &str
    ) {
        shader_module.load(self.backend(), name, code);
    }

    /*
    pub fn add_pipeline(&mut self, pipeline: Pipeline) -> Id<Pipeline> {
        self.last_pipeline_id += 1;
        let id = Id::new(self.last_pipeline_id);
        self.pipelines.insert(id, PipelineEntry{ pipeline, backend: None });
        self.pipelines_loaded = false;
        id
    }

    pub fn pipeline(&self, id: Id<Pipeline>) -> Option<&Pipeline> {
        self.pipelines.get(&id).map(|entry| &entry.pipeline)
    }

    pub fn pipeline_mut(&mut self, id: Id<Pipeline>) -> Option<&mut Pipeline> {
        self.pipelines.get_mut(&id).map(|entry| &mut entry.pipeline)
    }

    pub fn find_pipeline(&self, label: &str) -> Id<Pipeline> {
        for (id, entry) in self.pipelines.iter() {
            if entry.pipeline.label.eq(label) {
                return *id;
            }
        }
        return Id::default();
    }
    */

    pub fn reload(&mut self) {
        self.loaded = false;
    }

    pub fn bind(&mut self, pipeline: &mut Pipeline, layout: PipelineLayout) {
        let pipeline_backend = PipelineBackend::new(self.backend(), pipeline);
        let mut bindings = Bindings::default();
        bindings.load(self.backend(), &pipeline_backend, pipeline.bindings);
        pipeline.bindings = Some(bindings);
        self.pipelines.insert(pipeline.shader, pipeline_backend);
    }

    pub fn run(&mut self, pipeline: &mut Pipeline, mesh: &Mesh) {
        // TODO: it is not good to copy backend here, find another solution
        // let mut backend = self.backend.take();
        if let Some(pipeline_backend) = self.pipelines.get(&pipeline.shader) {
            pipeline_backend.run(
                self.backend(), //.as_mut().expect(RENDERER_STARTUP),
                &mesh.vertex_buffer,
                &pipeline.bindings
            );
        }
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
            pipelines: HashMap::new(),
            pipelines_loaded: false,
            last_pipeline_id: 0
        }
    }
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}


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

pub fn bind(mut renderer: Mut<Renderer>, mut assets: Mut<Assets>) {
    let clear_color = renderer.clear_color;
    renderer.backend_mut().bind_frame(&clear_color);

    if renderer.pipelines_loaded {
        return;
    }

    let mut pipelines_loaded = true;

    for (&id, shader) in assets.iter_mut::<Shader>() {
        shader.load(&renderer);
    }

    let backend = renderer.backend.take().unwrap();

    for entry in renderer.pipelines.values_mut() {
        // Try set default shader matching pipeline name if it was not set manually
        if entry.pipeline.shader.is_null() {
            if let Some(shader) = assets.find::<Shader>(&entry.pipeline.label) {
                entry.pipeline.shader = shader;
            }
        }

        let shader_id = entry.pipeline.shader;

        if entry.backend.is_none() {
            entry.backend = assets.get(shader_id).map(
                |shader| PipelineBackend::new(&backend, &entry.pipeline, &shader.module)
            ).or_else(|| {
                pipelines_loaded = false;
                None
            })
        }
    }
    renderer.backend = Some(backend);

    renderer.pipelines_loaded = pipelines_loaded;
}

pub fn release(mut renderer: Mut<Renderer>) {
    renderer.backend_mut().release_frame();
    renderer.cycle += 1;
    if renderer.cycle == 0 {
        renderer.cycle = 1;
    }
}

pub fn resize(mut renderer: Mut<Renderer>, window: Const<Window>) {
    let size = window.inner_size();
    renderer.backend_mut().resize(size.x, size.y);
}

#[derive(Default)]
pub struct PipelineLayout {
    pub label: String,
    pub mesh: &Mesh,
    pub bindings: &[BindGroupEntry],
    /// Depth buffer option
    pub use_depth_buffer: bool,
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

/// Binding types (Label, Stage, Buffer)
pub enum Binding<'a> {
    /// Uniform binding
    Uniform(String, Stage, &'a UniformBuffer),
    /// 2D Texture binding
    Texture(String, Stage, &'a TextureBuffer),
    // 3D Texture binding
    // Texture3d(To),
    /// Texture sampler binding
    Sampler(String, Stage, &'a Sampler),
}

pub struct BindGroupLayout {
    pub label: String,
    pub bindings: Vec<BindingType>,
}

pub struct BindingLayout {
    pub label: String,
    pub vertex: bool,
    pub fragment: bool,
    pub compute: bool,
}

pub enum Stage {
    Vertex,
    Fragment,
    Compute,
    All
}

pub struct BindGroup<'a> {
    pub bindings: Vec<Binding<'a>>
}

impl<'a> BindGroup<'a> {
    /*
    pub fn layout(label: &str, bindings: Vec<BindingType>) -> BindGroupLayout {
        BindGroupLayout {
            label: String::from(label),
            bindings,
        }
    }*/

    pub fn entry(bindings: Vec<BindingEntry<'a>>) -> BindGroup<'a> {
        BindGroup {
            bindings
        }
    }
}

    /*
/// System to render models, skyboxes, wire frames and overlays
pub fn world_renderer(
    mut ctx: Context<WorldRenderer>,
    mut renderer: Mut<Renderer>,
    mut assets: Mut<Assets>,
    camera: Const<Camera>,
    window: Const<Window>,
    world: Const<World>
) {
    if ctx.pipelines.is_none() {
        let skybox = renderer.add_skybox_pipeline();
        let static_model = renderer.add_static_model_pipeline();
        let skinned_model = renderer.add_skinned_model_pipeline();
        let overlay = renderer.add_overlay_pipeline();
        // let wire_frame = renderer.add_wire_frame_pipeline();

        ctx.pipelines = Some(
            Pipelines {
                skybox,
                static_model,
                skinned_model,
                overlay,
                // TODO: fix after shader rewriting
                wire_frame: Id::default(),
            }
        );
    }

    let device = &renderer.device;
    let queue = &renderer.queue;
    let depth_buffer = &renderer.depth_buffer;
    let frame = &renderer.frame()
        .expect("Frame should be created before the rendering cycle")
        .output;

    // Prepare sampler
    if ctx.sampler.is_none() {
        ctx.sampler = Some(device.create_sampler(&backend::SamplerDescriptor {
            address_mode_u: backend::AddressMode::Repeat,
            address_mode_v: backend::AddressMode::Repeat,
            address_mode_w: backend::AddressMode::Repeat,
            mag_filter: backend::FilterMode::Nearest,
            min_filter: backend::FilterMode::Linear,
            mipmap_filter: backend::FilterMode::Nearest,
            ..Default::default()
        }));
    }

    // Prepare lights
    let mut lights = LightUniform::default();

    // TODO: consider a single component for all lights
    let query = world.query::<(&AmbientLight,)>();
    for (amb_light,) in query {
        lights.ambient = amb_light.to_raw();
    }

    let query = world.query::<(&DirLight,)>();
    for (dir_light,) in query {
        if dir_light.enabled {
            lights.push_dir_light(dir_light.to_raw());
        }
    }

    let query = world.query::<(&PointLight,)>();
    for (point_light,) in query {
        if point_light.enabled {
            lights.push_point_light(point_light.to_raw());
        }
    }

    let query = world.query::<(&SimpleLight,)>();
    for (simple_light,) in query {
        if simple_light.enabled {
            lights.push_simple_light(simple_light.to_raw());
        }
    }

    let query = world.query::<(&SpotLight,)>();
    for (spot_light,) in query {
        if spot_light.enabled {
            lights.push_spot_light(spot_light.to_raw());
        }
    }

    if let Some(lights_buffer) = ctx.lights_buffer.as_ref() {
        queue.write_buffer(lights_buffer, 0, bytemuck::cast_slice(&[lights]));
    } else {
        ctx.lights_buffer = Some(device.create_buffer_init(
            &backend::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[lights]),
                usage: backend::BufferUsage::UNIFORM | backend::BufferUsage::COPY_DST,
            }
        ));
    }

    // Prepare projection * view matrix
    let proj_view_matrix = OPENGL_TO_WGPU_MATRIX * renderer.projection * camera.view();
    let proj_view_slice = AsRef::<[f32; 16]>::as_ref(&proj_view_matrix);

    if let Some(proj_view_buffer) = ctx.proj_view_buffer.as_ref() {
        queue.write_buffer(proj_view_buffer, 0, bytemuck::cast_slice(proj_view_slice));
    } else {
        ctx.proj_view_buffer = Some(device.create_buffer_init(&backend::util::BufferInitDescriptor {
            label: Some("ProjView Buffer"),
            contents: bytemuck::cast_slice(proj_view_slice),
            usage: backend::BufferUsage::UNIFORM | backend::BufferUsage::COPY_DST,
        }));
    }

    // prepare the command encoder and clean the surface
    let command_encoder_descriptor = backend::CommandEncoderDescriptor { label: None };
    let mut encoder = device.create_command_encoder(&command_encoder_descriptor);
    {
        encoder.begin_render_pass(&backend::RenderPassDescriptor {
            label: None,
            color_attachments: &[backend::RenderPassColorAttachment {
                view: &frame.view,
                resolve_target: None,
                ops: backend::Operations {
                    load: backend::LoadOp::Clear(renderer.clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(backend::RenderPassDepthStencilAttachment {
                view: depth_buffer,
                depth_ops: Some(backend::Operations {
                    load: backend::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
    }

    let sampler = ctx.sampler.as_ref().unwrap();

    // render skybox
    let query = world.query::<(&mut SkyBox,)>();
    for (skybox,) in query {
        if skybox.pipeline.is_null() {
            skybox.pipeline = ctx.pipelines.as_ref().unwrap().skybox;
        }
        let pipeline = renderer.pipeline(skybox.pipeline);
        let proj_view = OPENGL_TO_WGPU_MATRIX * renderer.projection * camera.view_static();

        skybox.load(&assets, device, queue, pipeline, sampler, &proj_view);
        skybox.draw(&mut encoder, pipeline, frame);
    }

    // render static models
    let query = world.query::<(&mut Model,)>();
    for (model,) in query {
        if model.disabled {
            continue;
        }

        if model.pipeline.is_null() {
            let pipelines = ctx.pipelines.as_ref().unwrap();
            model.pipeline = if !model.skin.is_null() {
                pipelines.skinned_model
            } else {
                pipelines.static_model
            };
        }
        let pipeline = renderer.pipeline(model.pipeline);
        let proj_view_buffer = ctx.proj_view_buffer.as_ref().unwrap();
        let lights_buffer = ctx.lights_buffer.as_ref().unwrap();

        model.load(&renderer, &mut assets, pipeline, sampler, proj_view_buffer, lights_buffer);
        model.draw(&assets, &mut encoder, pipeline, frame, depth_buffer);
    }

    let query = world.query::<(&mut WireFrame,)>();
    for (wire_frame,) in query {

        if wire_frame.disabled {
            continue;
        }

        if wire_frame.pipeline.is_null() {
            let pipelines = ctx.pipelines.as_ref().unwrap();
            wire_frame.pipeline = pipelines.wire_frame;
        }
        let pipeline = renderer.pipeline(wire_frame.pipeline);
        let proj_view_buffer = ctx.proj_view_buffer.as_ref().unwrap();

        wire_frame.load(&renderer, &mut assets, pipeline, proj_view_buffer);
        wire_frame.draw(&assets, &mut encoder, pipeline, frame, depth_buffer);
    }

    for overlay in &renderer.overlay {
        let scale_factor = window.scale_factor() as f32;
        let size = window.inner_size();

        for widget in &mut overlay.widgets(scale_factor, size.x as f32, size.y as f32) {
            if widget.pipeline.is_null() {
                widget.pipeline = ctx.pipelines.as_ref().unwrap().overlay;
            }
            let pipeline = renderer.pipeline(widget.pipeline);
            widget.load(&renderer, &mut assets, pipeline, sampler, &window);
            widget.draw(&mut encoder, pipeline, frame);
        }
    }

    // submit rendering
    queue.submit(Some(encoder.finish()));
}

    */


