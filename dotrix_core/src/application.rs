pub mod services;

use std::{
    time::{ Duration, Instant },
};

use winit::{
    event::{ Event, WindowEvent },
    event_loop::{ ControlFlow, EventLoop },
    window::{ Window, WindowBuilder }
};

use crate::{
    assets::Assets,
    ecs::System,
    frame::Frame,
    input::Input,
    renderer::Renderer,
    scheduler::Scheduler,
};

use services::Services;

pub struct Application {
    name: &'static str,
    scheduler: Scheduler,
    services: Services,
    clear_color: [f64; 4],
    fullscreen: bool,
}

impl Application {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            scheduler: Scheduler::new(),
            services: Services::new(),
            clear_color: [0.1, 0.2, 0.3, 1.0],
            fullscreen: false,
        }
    }

    pub fn set_display(&mut self, clear_color: [f64; 4], fullscreen: bool) {
        self.clear_color = clear_color;
        self.fullscreen = fullscreen;
    }

    pub fn add_system(&mut self, system: System) {
        self.scheduler.add(system);
    }

    pub fn add_service<T: Service>(&mut self, service: T) {
        self.services.add(service);
    }

    pub fn service<T: Service>(&mut self) -> &mut T
    {
        self.services.get_mut::<T>().expect("Application services does not exist")
    }

    /// Run the application
    pub fn run(self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(self.name)
            .build(&event_loop).unwrap();

        wgpu_subscriber::initialize_default_subscriber(None);

        run(event_loop, window, self);
    }
}

pub trait Service: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Service for T {}

/// Application run cycle
fn run(
    event_loop: EventLoop<()>,
    window: Window,
    Application {
        clear_color,
        mut scheduler,
        mut services,
        ..
    }: Application
) {
    // initalize WGPU and surface
    let (device, queue, surface) = futures::executor::block_on(init_surface(&window));

    let (mut pool, _spawner) = {
        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };
    let mut last_update_inst = Instant::now();

    services.add(Renderer::new(device, queue, surface, window, clear_color));

    scheduler.run_startup(&mut services);

    event_loop.run(move |event, _, control_flow| {
        // TODO: other possibilities?
        // *control_flow = ControlFlow::Poll;
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(10));

        if let Some(input) = services.get_mut::<Input>() {
            input.on_event(&event);
        }

        if let Some(assets) = services.get_mut::<Assets>() {
            assets.fetch();
        }

        match event {
            Event::MainEventsCleared => {
                if last_update_inst.elapsed() > Duration::from_millis(5) {
                    if let Some(renderer) = services.get_mut::<Renderer>() {
                        renderer.window.request_redraw();
                    }
                    last_update_inst = Instant::now();
                }
                pool.run_until_stalled();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Recreate the swap chain with the new size
                if let Some(renderer) = services.get_mut::<Renderer>() {
                    renderer.resize(size.width, size.height)
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::RedrawRequested(_) => {
                if let Some(frame) = services.get_mut::<Frame>() {
                    frame.next();
                }
                scheduler.run_standard(&mut services);
                if let Some(renderer) = services.get_mut::<Renderer>() {
                    renderer.next_frame();
                }
                scheduler.run_render(&mut services);
                if let Some(renderer) = services.get_mut::<Renderer>() {
                    renderer.finalize();
                }
                if let Some(input) = services.get_mut::<Input>() {
                    input.reset();
                }
            }
            _ => {}
        }
    });
}


async fn init_surface(
    window: &winit::window::Window
) -> (
    wgpu::Device,
    wgpu::Queue,
    wgpu::Surface,
) {
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropiate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None, // Some(&std::path::Path::new("./wgpu-trace/")),
        )
        .await
        .expect("Failed to create device");

    ( device, queue, surface )
}
