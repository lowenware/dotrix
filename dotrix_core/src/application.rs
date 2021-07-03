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
    window::Window,
    input::Input,
};

/// Application data to maintain the process
///
/// Do not construct it manually, use [`crate::Dotrix`] instead
pub struct Application {
    name: &'static str,
    scheduler: Scheduler,
    services: Services,
}

impl Application {
    /// Constructs new [`Application`] with defined name
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            scheduler: Scheduler::new(),
            services: Services::new(),
        }
    }

    /// Adds a system to the [`Application`]
    pub fn add_system(&mut self, system: System) {
        self.scheduler.add(system);
    }

    /// Adds a service to the [`Application`]
    pub fn add_service<T: IntoService>(&mut self, service: T) {
        self.services.add(service);
    }

    /// Returns a service of the [`Application`]
    pub fn service<T: IntoService>(&mut self) -> &mut T
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

/// Service wrapper
pub struct Service<T> {
    /// Service instance
    pub node: T
}

impl<T: IntoService> Service<T> {
    /// Wraps service data
    pub fn from(node: T) -> Self
    {
        Service {
            node
        }
    }
}

/// Service abstraction
///
/// More info about [`crate::services`]
pub trait IntoService: Sized + Send + Sync + 'static {
    /// Constructs wrapped service
    fn service(self) -> Service<Self> {
        Service {
            node: self
        }
    }
}
impl<T: Sized + Send + Sync + 'static> IntoService for T {}

/// Application run cycle
fn run(
    event_loop: EventLoop<()>,
    winit_window: WinitWindow,
    Application {
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
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
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

    pub(crate) fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub(crate) fn add<T: IntoService>(&mut self, service: T) {
        self.storage.insert(TypeId::of::<T>(), Box::new(service));
    }

    pub(crate) fn get<T: IntoService>(&self) -> Option<&T> {
        self.storage
            .get(&TypeId::of::<T>())
            .map(|srv| srv.downcast_ref::<T>().unwrap())
    }

    pub(crate) fn get_mut<T: IntoService>(&mut self) -> Option<&mut T> {
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

        let System { data, run_level } = system;

        let storage = match run_level {
            RunLevel::Render => &mut self.render,
            RunLevel::Standard => &mut self.standard,
            RunLevel::Startup => &mut self.startup,
            RunLevel::Bind => &mut self.bind,
            RunLevel::Release => &mut self.release,
            RunLevel::Resize => &mut self.resize,
        };

        storage.push(data);
        storage.sort_by(|s1, s2| {
            let p1: u32 = s1.priority().into();
            let p2: u32 = s2.priority().into();
            p2.cmp(&p1)
        });
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
