mod buffer;
mod pipeline;
mod shader;
mod texture;

use std::any::Any;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use dotrix::Output;
use dotrix_core as dotrix;
use dotrix_log as log;
use dotrix_types as types;
use dotrix_window as window;

use types::vertex;
use types::{Frame, Id};

pub use buffer::Buffer;
pub use pipeline::{PipelineLayout, RenderPipeline};
pub use shader::ShaderModule;
pub use texture::{Texture, TextureView};

pub use wgpu as backend;

const FPS_MEASURE_INTERVAL: u32 = 5; // seconds

pub struct Descriptor<'a> {
    pub window_handle: &'a window::Handle,
    pub fps_limit: Option<f32>,
    pub surface_size: [u32; 2],
    pub sample_count: u32,
}

pub struct Gpu {
    /// Sample Count
    sample_count: u32,
    /// Log of frames duration
    frames_duration: VecDeque<Duration>,
    /// Last frame timestamp
    last_frame: Option<Instant>,
    /// Real fps
    // fps: f32,
    /// WGPU Adapter
    adapter: wgpu::Adapter,
    /// WGPU Device
    device: wgpu::Device,
    /// WGPU Queue
    queue: wgpu::Queue,
    /// WGPU Surface
    surface: wgpu::Surface,
    /// WGPU surface configuration
    surface_conf: wgpu::SurfaceConfiguration,
    /// Surface resize request
    resize_request: Option<[u32; 2]>,
    /// Storage for GPU related objects: Buffers, Textures, Shaders, Pipelines, etc
    storage: HashMap<uuid::Uuid, Box<dyn Any>>,
    /// depth buffer view
    depth_buffer: TextureView,
    multisampled_framebuffer: TextureView,

    frame_texture: Option<(wgpu::SurfaceTexture, wgpu::TextureView)>,
    frame_number: u64,
}

pub struct CommandEncoder {
    pub inner: wgpu::CommandEncoder,
}

impl CommandEncoder {
    pub fn finish(mut self, priority: u32) -> Commands {
        Commands {
            inner: self.inner.finish(),
            priority,
        }
    }
}

pub struct Commands {
    pub priority: u32,
    pub inner: wgpu::CommandBuffer,
}

pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

/// Submit Report
///
/// As a task output identifies, that commands queue was executed
pub struct Submit {
    /// How long did it take to prepare the frame
    pub duration: Duration,
}

impl Gpu {
    pub fn new(descriptor: Descriptor) -> Self {
        let (adapter, device, queue, surface) =
            futures::executor::block_on(init(descriptor.window_handle));

        let surface_conf = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: descriptor.surface_size[0],
            height: descriptor.surface_size[1],
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &surface_conf);
        let sample_count = descriptor.sample_count;
        let fps_limit = descriptor.fps_limit.unwrap_or(240.0);
        let fps_samples = (FPS_MEASURE_INTERVAL * fps_limit.round() as u32) as usize;
        let frames_duration = VecDeque::with_capacity(fps_samples);
        let depth_buffer = create_depth_buffer(&device, &surface_conf, sample_count);
        let multisampled_framebuffer =
            create_multisampled_framebuffer(&device, &surface_conf, sample_count);

        Self {
            sample_count,
            frames_duration,
            last_frame: Some(Instant::now()),
            adapter,
            device,
            queue,
            surface,
            surface_conf,
            resize_request: None,
            storage: HashMap::new(),
            depth_buffer,
            multisampled_framebuffer,
            frame_texture: None,
            frame_number: 0,
        }
    }

    pub fn store<T: Any>(&mut self, data: T) -> Id<T> {
        let raw_id = uuid::Uuid::new_v4();
        self.storage.insert(raw_id, Box::new(data));
        Id::from(raw_id)
    }

    pub fn store_as<T: Any>(&mut self, id: Id<T>, data: T) {
        self.storage.insert(id.uuid().clone(), Box::new(data));
    }

    pub fn get<T: Any>(&self, id: &Id<T>) -> Option<&T> {
        self.storage
            .get(id.uuid())
            .and_then(|data| data.downcast_ref::<T>())
    }

    pub fn get_mut<T: Any>(&mut self, id: &Id<T>) -> Option<&mut T> {
        self.storage
            .get_mut(id.uuid())
            .and_then(|data| data.downcast_mut::<T>())
    }

    pub fn extract<T: Any>(&self, id: &Id<T>) -> &T {
        self.get(id).expect("Extraction of non-existing buffer")
    }

    pub fn buffer<'a, 'b>(&'a self, label: &'b str) -> buffer::Builder<'a, 'b> {
        buffer::Builder {
            gpu: self,
            descriptor: wgpu::BufferDescriptor {
                label: Some(label),
                usage: wgpu::BufferUsages::empty(),
                size: 0,
                mapped_at_creation: false,
            },
        }
    }

    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> Buffer {
        Buffer {
            inner: self.device.create_buffer(desc),
        }
    }

    pub fn write_buffer(&self, buffer: &Buffer, offset: u64, data: &[u8]) {
        self.queue.write_buffer(&buffer.inner, offset, data);
    }

    pub fn write_buffer_by_id(&self, id: &Id<Buffer>, offset: u64, data: &[u8]) {
        if let Some(buffer) = self.get(id) {
            self.queue.write_buffer(&buffer.inner, offset, data);
        }
    }

    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(desc)
    }

    pub fn create_bind_group(&self, desc: &wgpu::BindGroupDescriptor) -> wgpu::BindGroup {
        self.device.create_bind_group(&desc)
    }

    pub fn create_pipeline_layout(&self, desc: &wgpu::PipelineLayoutDescriptor) -> PipelineLayout {
        PipelineLayout {
            inner: self.device.create_pipeline_layout(desc),
        }
    }

    pub fn create_render_pipeline(&self, desc: &wgpu::RenderPipelineDescriptor) -> RenderPipeline {
        RenderPipeline {
            inner: self.device.create_render_pipeline(desc),
        }
    }

    pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> wgpu::Sampler {
        self.device.create_sampler(desc)
    }

    pub fn texture<'a, 'b>(&'a self, label: &'b str) -> texture::Builder<'a, 'b> {
        texture::Builder {
            gpu: self,
            descriptor: wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                format: wgpu::TextureFormat::Rgba8Uint,
                usage: wgpu::TextureUsages::empty(),
                dimension: wgpu::TextureDimension::D2,
            },
        }
    }

    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor) -> Texture {
        Texture {
            inner: self.device.create_texture(desc),
        }
    }

    pub fn create_shader_module(&self, name: &str, source: Cow<str>) -> ShaderModule {
        ShaderModule {
            inner: self
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(name),
                    source: wgpu::ShaderSource::Wgsl(source),
                }),
        }
    }

    pub fn encoder(&self, label: Option<&str>) -> CommandEncoder {
        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor { label };
        CommandEncoder {
            inner: self
                .device
                .create_command_encoder(&command_encoder_descriptor),
        }
    }

    pub fn resize_request(&mut self, width: u32, height: u32) {
        self.resize_request = Some([width, height]);
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_conf.format
    }

    pub fn depth_buffer(&self) -> &TextureView {
        &self.depth_buffer
    }

    pub fn color_attachment<'a>(
        &'a self,
    ) -> (&'a wgpu::TextureView, Option<&'a wgpu::TextureView>) {
        let view = self.frame_texture_view();
        if self.sample_count == 1 {
            (view, None)
        } else {
            (&self.multisampled_framebuffer.inner, Some(view))
        }
    }

    pub fn frame_texture_view(&self) -> &wgpu::TextureView {
        self.frame_texture
            .as_ref()
            .map(|(_, view)| view)
            .expect("Frame does not exists")
    }

    fn take_frame_output(&mut self) -> FrameOutput {
        let (surface_texture, view) = self.frame_texture.take().expect("Frame does not exists");
        FrameOutput {
            surface_texture,
            view,
        }
    }
    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn features(&self) -> wgpu::Features {
        self.device.features()
    }

    fn create_depth_buffer(&self) -> TextureView {
        create_depth_buffer(&self.device, &self.surface_conf, self.sample_count)
    }

    fn create_multisampled_framebuffer(&self) -> TextureView {
        create_multisampled_framebuffer(&self.device, &self.surface_conf, self.sample_count)
    }
}

pub struct FrameOutput {
    surface_texture: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

impl FrameOutput {
    pub fn present(self) {
        self.surface_texture.present();
    }
}

pub fn map_vertex_format(attr_format: vertex::AttributeFormat) -> wgpu::VertexFormat {
    match attr_format {
        vertex::AttributeFormat::Float32 => wgpu::VertexFormat::Float32,
        vertex::AttributeFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
        vertex::AttributeFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
        vertex::AttributeFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
        vertex::AttributeFormat::Uint16x2 => wgpu::VertexFormat::Uint16x2,
        vertex::AttributeFormat::Uint16x4 => wgpu::VertexFormat::Uint16x4,
        vertex::AttributeFormat::Uint32 => wgpu::VertexFormat::Uint32,
        vertex::AttributeFormat::Uint32x2 => wgpu::VertexFormat::Float32x2,
        vertex::AttributeFormat::Uint32x3 => wgpu::VertexFormat::Float32x3,
        vertex::AttributeFormat::Uint32x4 => wgpu::VertexFormat::Float32x4,
    }
}

pub struct CreateFrame {
    last_report: Instant,
    report_interval: Duration,
}

impl Default for CreateFrame {
    fn default() -> Self {
        Self {
            last_report: Instant::now(),
            report_interval: Duration::from_secs_f32(5.0),
        }
    }
}

impl dotrix::Task for CreateFrame {
    type Context = (dotrix::Mut<Gpu>,);
    type Output = Frame;

    fn run(&mut self, (mut renderer,): Self::Context) -> Self::Output {
        let delta = renderer
            .last_frame
            .replace(Instant::now())
            .map(|i| i.elapsed())
            .unwrap();

        if renderer.frames_duration.len() == renderer.frames_duration.capacity() {
            renderer.frames_duration.pop_back();
        }

        renderer.frames_duration.push_front(delta);

        let frames = renderer.frames_duration.len() as f32;
        let duration: f32 = renderer
            .frames_duration
            .iter()
            .map(|d| d.as_secs_f32())
            .sum();
        let fps = frames / duration;

        if let Some(resize_request) = renderer.resize_request.take() {
            let [width, height] = resize_request;
            if width > 0 && height > 0 {
                renderer.surface_conf.width = width;
                renderer.surface_conf.height = height;
                renderer
                    .surface
                    .configure(&renderer.device, &renderer.surface_conf);
                renderer.depth_buffer = renderer.create_depth_buffer();
                renderer.multisampled_framebuffer = renderer.create_multisampled_framebuffer();
            }
        }

        let wgpu_frame = match renderer.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                renderer
                    .surface
                    .configure(&renderer.device, &renderer.surface_conf);
                renderer
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture")
            }
        };

        let view = wgpu_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        renderer.frame_texture = Some((wgpu_frame, view));

        let frame_number = renderer.frame_number;
        renderer.frame_number += 1;

        if self.last_report.elapsed() > self.report_interval {
            log::info!("FPS: {:.02}", fps);
            self.last_report = Instant::now();
        }

        Frame {
            fps,
            delta,
            instant: Instant::now(),
            width: renderer.surface_conf.width,
            height: renderer.surface_conf.height,
            number: frame_number,
            scale_factor: 1.0, // TODO: implement
        }
    }
}

unsafe impl Send for Gpu {}
unsafe impl Sync for Gpu {}

#[derive(Default)]
pub struct ResizeSurface;

impl dotrix::Task for ResizeSurface {
    type Context = (dotrix::Take<dotrix::All<SurfaceSize>>, dotrix::Mut<Gpu>);
    type Output = ();

    fn run(&mut self, (mut sizes, mut renderer): Self::Context) -> Self::Output {
        if let Some(surface_size) = sizes.drain().last() {
            log::info!(
                "create surface resize request for: {}x{}",
                surface_size.width,
                surface_size.height
            );
            renderer.resize_request = Some([surface_size.width, surface_size.height]);
        }
    }
}

pub struct ClearFrame {
    color: types::Color<f32>,
}

impl Default for ClearFrame {
    fn default() -> Self {
        Self {
            color: types::Color::black(),
        }
    }
}

impl dotrix::Task for ClearFrame {
    type Context = (dotrix::Any<Frame>, dotrix::Ref<Gpu>);
    // The task uses itself as output as a zero-cost abstraction
    type Output = Commands;
    fn run(&mut self, (_, renderer): Self::Context) -> Self::Output {
        let mut encoder = renderer.encoder(Some("dotrix::gpu::clear_frame"));

        let clear_color = wgpu::Color {
            r: self.color.r as f64,
            g: self.color.g as f64,
            b: self.color.b as f64,
            a: self.color.a as f64,
        };

        let (view, resolve_target) = renderer.color_attachment();

        let rpass_color_attachment = wgpu::RenderPassColorAttachment {
            view,
            resolve_target,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
                store: true,
            },
        };

        encoder
            .inner
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(rpass_color_attachment)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &renderer.depth_buffer.inner,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

        encoder.finish(1000)
    }
}

#[derive(Default)]
pub struct SubmitCommands;

impl dotrix::Task for SubmitCommands {
    type Context = (
        dotrix::Any<Frame>,
        dotrix::Take<dotrix::All<Commands>>,
        dotrix::Mut<Gpu>,
    );

    fn output_channel(&self) -> dotrix::task::OutputChannel {
        dotrix::task::OutputChannel::Scheduler
    }

    // The task uses itself as output as a zero-cost abstraction
    type Output = FrameOutput;
    fn run(&mut self, (_, commands, mut gpu): Self::Context) -> Self::Output {
        let mut commands = commands.take();

        commands.sort_by(|a, b| a.priority.cmp(&b.priority));

        // for c in commands.iter() {
        //     log::debug!("Commands: {}", c.priority);
        // }

        let index = gpu.queue.submit(commands.into_iter().map(|c| c.inner));

        while !gpu
            .device
            .poll(wgpu::Maintain::WaitForSubmissionIndex(index))
        {}

        gpu.take_frame_output()
    }
}

async fn init(
    window_handle: &window::Handle,
) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue, wgpu::Surface) {
    let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);

    let instance = wgpu::Instance::new(backend);
    let surface = unsafe { instance.create_surface(&window_handle) };
    let adapter =
        wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
            .await
            .expect("No suitable GPU adapters found on the system!");

    #[cfg(not(target_arch = "wasm32"))]
    {
        let adapter_info = adapter.get_info();
        log::info!("Adapter: {}", adapter_info.name);
        log::info!("Backend: {:?}", adapter_info.backend);
    }

    // TODO: implement features control
    let optional_features = wgpu::Features::empty();
    let required_features =
        wgpu::Features::MULTI_DRAW_INDIRECT | wgpu::Features::INDIRECT_FIRST_INSTANCE;
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Not supported: {:?}",
        required_features - adapter_features
    );

    let required_downlevel_capabilities = wgpu::DownlevelCapabilities {
        flags: wgpu::DownlevelFlags::empty(),
        shader_model: wgpu::ShaderModel::Sm5,
        ..wgpu::DownlevelCapabilities::default()
    };
    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    assert!(
        downlevel_capabilities.shader_model >= required_downlevel_capabilities.shader_model,
        "Shader model {:?} requiered, but {:?} supported ",
        required_downlevel_capabilities.shader_model,
        downlevel_capabilities.shader_model,
    );
    assert!(
        downlevel_capabilities
            .flags
            .contains(required_downlevel_capabilities.flags),
        "Adapter does not support the downlevel capabilities required to run: {:?}",
        required_downlevel_capabilities.flags - downlevel_capabilities.flags
    );

    // Make sure we use the texture resolution limits from the adapter, so we can support images
    // the size of the surface.
    let mut gpu_limits =
        wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
    gpu_limits.max_storage_buffers_per_shader_stage = 5;
    gpu_limits.max_storage_buffer_binding_size = 1 * 1024 * 1024 * 1024;

    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: (optional_features & adapter_features) | required_features,
                limits: gpu_limits,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Unable to find a suitable GPU adapter!");

    (adapter, device, queue, surface)
}

fn create_depth_buffer(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
) -> TextureView {
    let buffer_extent = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let texture = wgpu::TextureDescriptor {
        label: Some("dotrix::gpu::depth_buffer"),
        size: buffer_extent,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
    };

    TextureView {
        inner: device
            .create_texture(&texture)
            .create_view(&wgpu::TextureViewDescriptor::default()),
    }
}

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
) -> TextureView {
    let multisampled_texture_extent = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        size: multisampled_texture_extent,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("dotrix::gpu::multisampled_framebuffer"),
    };

    TextureView {
        inner: device
            .create_texture(multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
