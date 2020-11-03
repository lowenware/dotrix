use std::{
    any::TypeId,
    collections::HashMap,
    time::{Duration, Instant},
};

use winit::{
    event::{ Event, WindowEvent },
    event_loop::{ ControlFlow, EventLoop },
    window::{ Window, WindowBuilder }
};

use crate::{
    ecs::{System},
    input::{InputConfig, InputManager},
    renderer::Renderer,
    scheduler::Scheduler,
    services::{Services, Service},
};

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



/*
trait Accessor {
    type Item;
    fn clone(&self) -> Self::Item;
}

impl Accessor for 
*/


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

    let renderer = futures::executor::block_on(Renderer::new(&window));

    let mut swap_chain = renderer.swap_chain();
    services.add(renderer);

    let (mut pool, spawner) = {
        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };
    let mut last_update_inst = Instant::now();

    // app.scheduler.run_startup(app);

    let mut input_manager = InputManager::new();
    let input_config = InputConfig::default();
    input_manager.initialize(&input_config);

    println!("{}", serde_json::to_string_pretty(&input_manager.create_config()).unwrap());

    event_loop.run(move |event, _, control_flow| {
        // TODO: other possibilities?
        *control_flow = ControlFlow::Poll;

        input_manager.update(); // TODO: can winit event loop runs more than once per frame?

        // app.scheduler.run_standard(app);

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
                let mut renderer = services.get_mut::<Renderer>();
                // Recreate the swap chain with the new size
                renderer.resize(size.width, size.height);
                swap_chain = renderer.swap_chain();
            }
            Event::RedrawRequested(_) => {
                /*
                let frame = match swap_chain.get_current_frame() {
                    Ok(frame) => frame,
                    Err(_) => {
                        swap_chain = device.create_swap_chain(&surface, &sc_desc);
                        swap_chain
                            .get_current_frame()
                            .expect("Failed to acquire next swap chain texture!")
                    }
                };

                example.render(&frame.output, &device, &queue, &spawner);
                */

                scheduler.run_render(&mut services);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput{device_id, input, is_synthetic},
                ..
            } => input_manager.handle_keyboard_event(device_id, input, is_synthetic),
            Event::WindowEvent {
                event: WindowEvent::MouseInput{device_id, state, button, modifiers},
                ..
            } => input_manager.handle_mouse_event(device_id, state, button),
            Event::WindowEvent {
                event: WindowEvent::MouseWheel{device_id, delta, phase, modifiers},
                ..
            } => input_manager.handle_mouse_wheel_event(device_id, delta, phase),
            _ => {}
        }
    });
}

