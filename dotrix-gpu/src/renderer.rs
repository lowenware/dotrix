use dotrix_core as dotrix;
use dotrix_log as log;
use dotrix_types as types;
use dotrix_window as window;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const FPS_MEASURE_INTERVAL: u32 = 5; // seconds
pub struct Descriptor<'a> {
    pub window_handle: &'a window::Handle,
    pub fps_request: f32,
    pub surface_size: [u32; 2],
    pub sample_count: u32,
}

pub struct Renderer {
    /// Desired FPS
    fps_request: f32,
    /// Sample Count
    sample_count: u32,
    /// Log of frames duration
    frames_duration: VecDeque<Duration>,
    /// Last frame timestamp
    last_frame: Option<Instant>,
    /// Real fps
    fps: f32,
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
}

pub struct Frame {
    pub inner: wgpu::SurfaceTexture,
    pub delta: std::time::Duration,
    pub instant: std::time::Instant,
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

impl Renderer {
    pub fn new(descriptor: Descriptor) -> Self {
        let (adapter, device, queue, surface) =
            futures::executor::block_on(init(descriptor.window_handle));

        let surface_conf = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: descriptor.surface_size[0],
            height: descriptor.surface_size[1],
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &surface_conf);
        let sample_count = descriptor.sample_count;
        let fps_request = descriptor.fps_request;
        let frame_duration = Duration::from_secs_f32(1.0 / fps_request);
        let fps_samples = (FPS_MEASURE_INTERVAL * fps_request.ceil() as u32) as usize;
        let mut frames_duration = VecDeque::with_capacity(fps_samples);

        Self {
            fps_request,
            sample_count,
            frames_duration,
            fps: fps_request,
            last_frame: None,
            adapter,
            device,
            queue,
            surface,
            surface_conf,
            resize_request: None,
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

    /*
    pub fn clear(&self) {


        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor { label: None };
        let mut encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);
        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                // We still need to use the depth buffer here
                // since the pipeline requires it.
                depth_stencil_attachment: None,
            });
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
    */
}

impl Frame {
    pub fn delta(&self) -> Duration {
        self.delta
    }
}

#[derive(Default)]
pub struct CreateFrame;

impl dotrix::Task for CreateFrame {
    type Context = (dotrix::Mut<Renderer>,);
    type Output = Frame;

    fn run(&mut self, (mut renderer,): Self::Context) -> Self::Output {
        let delta = renderer
            .last_frame
            .replace(Instant::now())
            .map(|i| i.elapsed())
            .unwrap_or_else(|| Duration::from_secs_f32(1.0 / renderer.fps_request));

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

        renderer.fps = fps;

        if let Some(resize_request) = renderer.resize_request.take() {
            let [width, height] = resize_request;
            if width > 0 && height > 0 {
                renderer.surface_conf.width = width;
                renderer.surface_conf.height = height;
                renderer
                    .surface
                    .configure(&renderer.device, &renderer.surface_conf);
            }
        }

        Frame {
            inner: match renderer.surface.get_current_texture() {
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
            },
            delta,
            instant: Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct ResizeSurface;

impl dotrix::Task for ResizeSurface {
    type Context = (dotrix::Take<SurfaceSize>, dotrix::Mut<Renderer>);
    type Output = ();

    fn run(&mut self, (surface_size, mut renderer): Self::Context) -> Self::Output {
        log::info!(
            "create surface resize request for: {}x{}",
            surface_size.width,
            surface_size.height
        );
        renderer.resize_request = Some([surface_size.width, surface_size.height]);
    }
}

pub struct ClearFrame {
    color: types::Color,
}

impl Default for ClearFrame {
    fn default() -> Self {
        Self {
            color: types::Color::black(),
        }
    }
}

impl dotrix::Task for ClearFrame {
    type Context = (dotrix::Any<Frame>, dotrix::Ref<Renderer>);
    // The task uses itself as output as a zero-cost abstraction
    type Output = Commands;
    fn run(&mut self, (frame, renderer): Self::Context) -> Self::Output {
        let view = frame
            .inner
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = renderer.encoder(Some("ClearFrame"));
        encoder
            .inner
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.color.r as f64,
                            g: self.color.g as f64,
                            b: self.color.b as f64,
                            a: self.color.a as f64,
                        }),
                        store: true,
                    },
                })],
                // We still need to use the depth buffer here
                // since the pipeline requires it.
                depth_stencil_attachment: None,
            });

        encoder.finish(0)
    }
}

#[derive(Default)]
pub struct SubmitCommands;

impl dotrix::Task for SubmitCommands {
    type Context = (
        dotrix::Any<Frame>,
        dotrix::Collect<Commands>,
        dotrix::Ref<Renderer>,
    );
    // The task uses itself as output as a zero-cost abstraction
    type Output = SubmitCommands;
    fn run(&mut self, (frame, commands, renderer): Self::Context) -> Self::Output {
        let index = renderer
            .queue
            .submit(commands.collect().into_iter().map(|c| c.inner));
        while !renderer
            .device
            .poll(wgpu::Maintain::WaitForSubmissionIndex(index))
        {}
        SubmitCommands
    }
}

#[derive(Default)]
pub struct PresentFrame;

impl dotrix::Task for PresentFrame {
    type Context = (dotrix::Take<Frame>, dotrix::Take<SubmitCommands>);

    type Output = PresentFrame;

    fn output_channel(&self) -> dotrix::task::OutputChannel {
        dotrix::task::OutputChannel::Scheduler
    }

    fn run(&mut self, (frame, _): Self::Context) -> Self::Output {
        frame.unwrap().inner.present();
        PresentFrame
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
    let required_features = wgpu::Features::empty();
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

    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the surface.
    let needed_limits =
        wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());

    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: (optional_features & adapter_features) | required_features,
                limits: needed_limits,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Unable to find a suitable GPU adapter!");

    (adapter, device, queue, surface)
}
