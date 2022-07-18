use dotrix_core as dotrix;
use dotrix_window as window;

pub struct RendererOptions {
    pub surface_size: [u32; 2],
    pub sample_count: u32,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            surface_size: [640, 480],
            sample_count: 2,
        }
    }
}

pub struct Renderer {
    options: RendererOptions,
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
}

impl Renderer {
    pub fn new(window_handle: &window::Handle, options: RendererOptions) -> Self {
        let (adapter, device, queue, surface) = futures::executor::block_on(init(window_handle));

        let surface_conf = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: options.surface_size[0],
            height: options.surface_size[1],
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &surface_conf);

        Self {
            options,
            adapter,
            device,
            queue,
            surface,
            surface_conf,
        }
    }

    pub fn clear(&self) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.surface_conf);
                self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture")
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor { label: None };
        let mut encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
}

async fn init(
    window_handle: &window::Handle,
) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue, wgpu::Surface) {
    wgpu_subscriber::initialize_default_subscriber(None);

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
        println!("Adapter: {}", adapter_info.name);
        println!("Backend: {:?}", adapter_info.backend);
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

    (
        adapter, device, queue, surface,
        // sur_desc,
    )
}
