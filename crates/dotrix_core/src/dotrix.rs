use crate::{
    window::Window,
};

use dotrix_ecs::{
    System, World
};

use winit::{
    event::{ Event, WindowEvent, },
    event_loop::{ ControlFlow, EventLoop},
    window::WindowBuilder,
};

/// The core structure of the Engine which methods allow to initialize systems and controllers
/// and start routines
pub struct Dotrix {
    window: Option<Window>,
    /// Systems container
    systems: Vec<Box<dyn System>>,
    world: World,
}

impl Dotrix {
    /// Initialize the core structure
    pub fn init() -> Self {
        Self {
            window: None,
            systems: Vec::new(),
            world: World::new(),
        }
    }

    /// Initialize application window
    pub fn window(&mut self, window: Window) -> &mut Self {
        self.window = Some(window);
        self
    }

    /// Run the application
    pub fn run(&mut self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        wgpu_subscriber::initialize_default_subscriber(None);

        futures::executor::block_on(run(event_loop, window, self));
    }

    /// Register a system
    pub fn add_system<T>(&mut self, system: T)
    where
        T: System
    {
        self.systems.push(Box::<T>::new(system));
    }

    /// start systems
    fn start_systems(&mut self) {
        for system in self.systems.iter_mut() {
            system.as_mut().start(&mut self.world);
        }
    }

    /// run systems
    fn run_systems(&mut self) {
        for system in self.systems.iter_mut() {
            system.as_mut().run(&mut self.world);
        }
    }
}



/// Application run cycle
async fn run(event_loop: EventLoop<()>, window: winit::window::Window, dotrix: &mut Dotrix) {
    let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropiate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Load the shaders from disk
    let vs_module = device.create_shader_module(wgpu::include_spirv!("../shader.vert.spv"));
    let fs_module = device.create_shader_module(wgpu::include_spirv!("../shader.frag.spv"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        // Use the default rasterizer state: no culling, no depth bias
        rasterization_state: None,
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[swapchain_format.into()],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (
            &instance,
            &adapter,
            &vs_module,
            &fs_module,
            &pipeline_layout,
        );

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Recreate the swap chain with the new size
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            }
            Event::RedrawRequested(_) => {
                // TODO: call 
                // * all systems run cycle
                // * world.render() here
                let frame = swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture")
                    .output;
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
