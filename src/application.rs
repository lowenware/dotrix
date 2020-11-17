pub mod services;

use std::{
    time::{Duration, Instant},
};

use winit::{
    event::{ Event, WindowEvent },
    event_loop::{ ControlFlow, EventLoop },
    window::{ Window, WindowBuilder }
};

use crate::{
    assets::Assets,
    ecs::{System},
    input::{InputConfig, InputManager},
    renderer::Renderer,
    scheduler::Scheduler,
};

use services::Services;

pub struct Application {
    name: &'static str,
    scheduler: Scheduler,
    services: Services,
}

impl Application {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            scheduler: Scheduler::new(),
            services: Services::new(),
        }
    }

    pub fn add_system(&mut self, system: System) {
        self.scheduler.add(system);
    }

    pub fn add_service<T: Service>(&mut self, service: T) {
        self.services.add(service);
    }

    pub fn service<T: Service>(&mut self) -> &mut T
    {
        self.services.get_mut::<T>()
    }

    /// Run the application
    pub fn run(self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

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
        name,
        mut scheduler, 
        mut services,
    }: Application
) {
    println!("Starting {}", name);

    let renderer = futures::executor::block_on(Renderer::new(&window));

    // let mut swap_chain = renderer.swap_chain();
    services.add(renderer);

    let (mut pool, _spawner) = {
        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };
    let mut last_update_inst = Instant::now();

    scheduler.run_startup(&mut services);

    services.add(InputManager::new());
    let input_config = InputConfig::default();
    services.get_mut::<InputManager>().initialize(&input_config);

    // println!("{}", serde_json::to_string_pretty(&input_manager.create_config()).unwrap());

    event_loop.run(move |event, _, control_flow| {
        // TODO: other possibilities?
        // *control_flow = ControlFlow::Poll;
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(10));

        services.get_mut::<InputManager>().update(); // TODO: can winit event loop runs more than once per frame?

        scheduler.run_standard(&mut services);
        services.get_mut::<Assets>().fetch();

        match event {
            Event::MainEventsCleared => {
                if last_update_inst.elapsed() > Duration::from_millis(20) {
                    window.request_redraw();
                    last_update_inst = Instant::now();
                }

                pool.run_until_stalled();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Recreate the swap chain with the new size
                services.get_mut::<Renderer>().resize(size.width, size.height);
            }
            Event::RedrawRequested(_) => {
                services.get_mut::<Renderer>().next_frame();
                scheduler.run_render(&mut services);
                services.get_mut::<Renderer>().finalize();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput{device_id, input, is_synthetic},
                ..
            } => services.get_mut::<InputManager>().handle_keyboard_event(device_id, input, is_synthetic),
            Event::WindowEvent {
                event: WindowEvent::MouseInput{device_id, state, button, ..},
                ..
            } => services.get_mut::<InputManager>().handle_mouse_event(device_id, state, button),
            Event::WindowEvent {
                event: WindowEvent::MouseWheel{device_id, delta, phase, ..},
                ..
            } => services.get_mut::<InputManager>().handle_mouse_wheel_event(device_id, delta, phase),
            _ => {}
        }
    });
}

