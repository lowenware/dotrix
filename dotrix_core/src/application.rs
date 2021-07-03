use std::{
    any::TypeId,
    collections::HashMap,
    time::{ Duration, Instant },
};

use winit::{
    event::{ Event, WindowEvent },
    event_loop::{ ControlFlow, EventLoop },
    window::{ Window as WinitWindow, WindowBuilder }
};

use crate::{
    assets::Assets,
    ecs::{ RunLevel, System, Systemized },
    frame::Frame,
    input::Input,
    renderer::Renderer,
    window::Window,
};

/// Application data to maintain the process
///
/// Do not construct it manually, use [`crate::Dotrix`] instead
pub struct Application {
    name: &'static str,
    scheduler: Scheduler,
    services: Services,
    clear_color: [f64; 4],
    fullscreen: bool,
}

impl Application {
    /// Constructs new [`Application`] with defined name
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            scheduler: Scheduler::new(),
            services: Services::new(),
            clear_color: [0.1, 0.2, 0.3, 1.0],
            fullscreen: false,
        }
    }

    /// Sets parameters for rendering output
    pub fn set_display(&mut self, clear_color: [f64; 4], fullscreen: bool) {
        self.clear_color = clear_color;
        self.fullscreen = fullscreen;
    }

    /// Adds a system to the [`Application`]
    pub fn add_system(&mut self, system: System) {
        self.scheduler.add(system);
    }

    /// Adds a service to the [`Application`]
    pub fn add_service<T: Service>(&mut self, service: T) {
        self.services.add(service);
    }

    /// Returns a service of the [`Application`]
    pub fn service<T: Service>(&mut self) -> &mut T
    {
        self.services.get_mut::<T>().expect("Application services does not exist")
    }

    /// Run the application
    pub fn run(self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .build(&event_loop).unwrap();

        wgpu_subscriber::initialize_default_subscriber(None);

        run(event_loop, window, self);
    }
}

/// Service abstraction
///
/// More info about [`crate::services`]
pub trait Service: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Service for T {}

/// Application run cycle
fn run(
    event_loop: EventLoop<()>,
    winit_window: WinitWindow,
    Application {
        clear_color,
        name,
        mut scheduler,
        mut services,
        ..
    }: Application
) {

    let (mut pool, _spawner) = {
        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };
    let mut last_update_inst = Instant::now();

    // !!! DO NOT CREATE SERVICES HERE !!!

    if let Some(window) = services.get_mut::<Window>() {
        window.set(winit_window);
        window.set_title(name);
    }

    scheduler.run_startup(&mut services);

    event_loop.run(move |event, _, control_flow| {
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
                    if let Some(window) = services.get::<Window>() {
                        if window.close_request {
                            *control_flow = ControlFlow::Exit;
                        } else {
                            window.request_redraw();
                        }
                    }
                    last_update_inst = Instant::now();
                }
                pool.run_until_stalled();
            }
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                // Recreate the swap chain with the new size
                // if let Some(renderer) = services.get_mut::<Renderer>() {
                //     renderer.resize(size.width, size.height)
                // }
                if let Some(window) = services.get_mut::<Window>() {
                    println!(
                        "Event Resize {}x{} vs Window {:?}",
                        size.width,
                        size.height,
                        window.inner_size()
                    );
                }
                scheduler.run_resize(&mut services);
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::RedrawRequested(_) => {
                // if let Some(frame) = services.get_mut::<Frame>() {
                //    frame.next();
                // }
                scheduler.run_bind(&mut services);
                scheduler.run_standard(&mut services);
                // if let Some(renderer) = services.get_mut::<Renderer>() {
                //     renderer.next_frame();
                // }
                scheduler.run_render(&mut services);
                scheduler.run_release(&mut services);
                // if let Some(renderer) = services.get_mut::<Renderer>() {
                //     renderer.finalize();
                // }
                // if let Some(input) = services.get_mut::<Input>() {
                //     input.reset();
                // }
            }
            _ => {}
        }
    });
}

/// Services manager
pub struct Services {
    storage: HashMap<TypeId, Box<dyn std::any::Any>>,
}

impl Services {

    fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    fn add<T: Service>(&mut self, service: T) {
        self.storage.insert(TypeId::of::<T>(), Box::new(service));
    }

    pub(crate) fn get<T: Service>(&self) -> Option<&T> {
        self.storage
            .get(&TypeId::of::<T>())
            .map(|srv| srv.downcast_ref::<T>().unwrap())
    }

    pub(crate) fn get_mut<T: Service>(&mut self) -> Option<&mut T> {
        self.storage
            .get_mut(&TypeId::of::<T>())
            .map(|srv| srv.downcast_mut::<T>().unwrap())
    }
}

/// Systems scheduler
struct Scheduler {
    render: Vec<Box<dyn Systemized>>,
    standard: Vec<Box<dyn Systemized>>,
    startup: Vec<Box<dyn Systemized>>,
    bind: Vec<Box<dyn Systemized>>,
    release: Vec<Box<dyn Systemized>>,
    resize: Vec<Box<dyn Systemized>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            render: Vec::new(),
            standard: Vec::new(),
            startup: Vec::new(),
            bind: Vec::new(),
            release: Vec::new(),
            resize: Vec::new(),
        }
    }

    pub fn add(&mut self, system: System) {
        let (data, run_level) = system.tuple();
        match run_level {
            RunLevel::Render => self.render.push(data),
            RunLevel::Standard => self.standard.push(data),
            RunLevel::Startup => self.startup.push(data),
            RunLevel::Bind => self.bind.push(data),
            RunLevel::Release => self.release.push(data),
            RunLevel::Resize => self.resize.push(data),
        };
    }

    pub fn run_render(&mut self, services: &mut Services) {
        for system in &mut self.render {
            system.run(services);
        }
    }

    pub fn run_standard(&mut self, services: &mut Services) {
        for system in &mut self.standard {
            system.run(services);
        }
    }

    pub fn run_startup(&mut self, services: &mut Services) {
        for system in &mut self.startup {
            system.run(services);
        }
    }

    pub fn run_bind(&mut self, services: &mut Services) {
        for system in &mut self.bind {
            system.run(services);
        }
    }

    pub fn run_release(&mut self, services: &mut Services) {
        for system in &mut self.release {
            system.run(services);
        }
    }

    pub fn run_resize(&mut self, services: &mut Services) {
        for system in &mut self.resize {
            system.run(services);
        }
    }
}
