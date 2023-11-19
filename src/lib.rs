//! Dotrix is a 3D game engine following ECS programming pattern with a goal to be simple and
//! feature rich. There is a [Löwenware](https://lowenware.com) team behind the project and we
//! are working on Dotrix to power up our own game projects.
//!
//! The best place to see what can be done with Dotrix is our
//! [YouTube](https://www.youtube.com/channel/UCdriNXRizbBFQhqZefaw44A) channel.
//!
//! ## Getting Started
//!
//! If you are more into a practice and looking for an example code, we've prepared a good
//! [demo application](https://github.com/lowenware/dotrix/blob/main/examples/demo/demo.rs) for you
//! to get started.
//!

#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
//#![warn(missing_docs)]

/// Logging utilities
pub mod log;
pub use crate::log::Log;

/// Rendering tools and routines
pub mod render;
pub use render::{DeviceType, Display, Extent2D, Frame, Gpu, Renderer, Semaphore};

/// Tasks and execution
pub mod tasks;
pub use tasks::{All, Any, Mut, Output, Ref, State, Take, Task, TaskManager};

/// Window API and input events
pub mod window;
pub use window::Window;

/// Utils
pub mod utils;

//pub use utils::{ Id };

// pub use dotrix_core::{All, Any, Extension, Manager, Mut, Output, Ref, State, Take, Task};
// pub use dotrix_types::{camera, type_lock, vertex, Color, Frame, Id, Transform};

// pub use dotrix_assets as assets;
// pub use dotrix_ecs as ecs;
// pub use dotrix_gpu as gpu;
// pub use dotrix_image as image;
// pub use dotrix_input as input;
// pub use dotrix_log as log;
// pub use dotrix_math as math;
// pub use dotrix_mesh as mesh;
// pub use dotrix_shader as shader;
// pub use dotrix_window as window;

// pub use assets::Assets;
// pub use camera::Camera;
// pub use ecs::World;
// use gpu::ResizeSurface;
// pub use input::Input;
// pub use mesh::{Armature, Mesh};
// pub use shader::Shader;
// pub use vertex::{Bitangent, Normal, Position, Tangent, TexUV};

// #[cfg(feature = "pbr")]
// pub use dotrix_pbr as pbr;

// #[cfg(feature = "ui")]
// pub use dotrix_ui as ui;

/// Dotrix Settings
pub struct CoreSetup<'a> {
    /// Number of workers
    pub workers: u32,
    /// FPS preference
    pub fps_request: Option<f32>,
    /// Log fps within an interval
    pub log_fps_interval: Option<std::time::Duration>,
    /// Application name
    pub app_name: &'a str,
    /// Application version
    pub app_version: u32,
    /// Debug flag
    pub debug: bool,
    /// Device type request
    pub device_type_request: Option<DeviceType>,
}

impl<'a> Default for CoreSetup<'a> {
    fn default() -> Self {
        Self {
            workers: 8,
            fps_request: None,
            log_fps_interval: None,
            app_name: env!("CARGO_PKG_NAME"),
            app_version: 0,
            debug: false,
            device_type_request: None,
        }
    }
}

impl<'a> CoreSetup<'a> {
    /// Sets number of workers
    pub fn workers(mut self, value: u32) -> Self {
        self.workers = value;
        self
    }

    /// Sets FPS preference
    pub fn fps_request(mut self, value: Option<f32>) -> Self {
        self.fps_request = value;
        self
    }

    /// Sets FPS logging
    pub fn log_fps_interval(mut self, value: Option<std::time::Duration>) -> Self {
        self.log_fps_interval = value;
        self
    }

    /// Sets application name for rendering backend
    pub fn application_name(mut self, app_name: &'a str) -> Self {
        self.app_name = app_name;
        self
    }

    /// Sets application version for rendering backend
    pub fn application_version(mut self, app_version: u32) -> Self {
        self.app_version = app_version;
        self
    }

    /// Enables debug mode for rendering backend
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Sets device type preference
    pub fn device_type_request(mut self, device_type_request: Option<DeviceType>) -> Self {
        self.device_type_request = device_type_request;
        self
    }

    pub fn create(self) -> Core {
        let resolution = Extent2D {
            width: 800,
            height: 600,
        };
        let fullscreen = false;
        let (window, window_event_loop) = Window::new(self.app_name, resolution, fullscreen);

        let display_setup = render::DisplaySetup {
            window,
            app_name: self.app_name,
            app_version: self.app_version,
            debug: self.debug,
            device_type_request: self.device_type_request,
        };

        let display = Display::new(display_setup);
        let gpu = display.gpu();

        let event_loop = EventLoop {
            window_event_loop,
            workers: self.workers,
            fps_request: self.fps_request,
            log_fps_interval: self.log_fps_interval,
        };

        Core {
            display,
            gpu,
            event_loop,
        }
    }
}

pub struct EventLoop {
    window_event_loop: window::EventLoop,
    workers: u32,
    fps_request: Option<f32>,
    log_fps_interval: Option<std::time::Duration>,
}

pub struct Core {
    pub display: Display,
    pub gpu: Gpu,
    pub event_loop: EventLoop,
}

impl Core {
    pub fn setup<'a>() -> CoreSetup<'a> {
        CoreSetup::default()
    }

    pub fn into_tuple(self) -> (Display, Gpu, EventLoop) {
        self.into()
    }
}

impl From<Core> for (Display, Gpu, EventLoop) {
    fn from(core: Core) -> Self {
        (core.display, core.gpu, core.event_loop)
    }
}

/// Application launcher
///
/// Inicializes window application with `Settings`, `Gpu` and provides a setup callback
pub fn run<F>(event_loop: EventLoop, setup: F)
where
    F: FnOnce(&tasks::Scheduler) + 'static,
{
    let EventLoop {
        mut window_event_loop,
        workers,
        fps_request,
        log_fps_interval,
    } = event_loop;

    let frame_duration = std::time::Duration::from_secs_f32(
        fps_request
            .as_ref()
            .map(|fps_request| 1.0 / fps_request)
            .unwrap_or(0.0),
    );

    window_event_loop.set_frame_duration(frame_duration);

    // Set target output, so scheduler can build the dependency graph
    let task_manager = TaskManager::new::<render::FramePresenter>(workers);
    {
        let scheduler = task_manager.scheduler();

        let create_frame_task = render::CreateFrame::default()
            .log_fps_interval(log_fps_interval)
            .fps_request(fps_request);
        scheduler.add_task(create_frame_task);

        let submit_frame_task = render::SubmitFrame::default();
        scheduler.add_task(submit_frame_task);

        setup(&scheduler);
        // self.task_manager.schedule(gpu::ClearFrame::default());
        // self.task_manager.schedule(gpu::SubmitCommands::default());
        // self.task_manager.schedule(gpu::ResizeSurface::default());

        // Input listening
        // self.task_manager.schedule(input::ListenTask::default());

        // register data provided by window controller
        // self.task_manager.register::<window::ResizeRequest>(0);
        // self.task_manager.register::<input::Event>(0);
    }

    let event_handler = WindowEventHandler::new(task_manager);

    window_event_loop.run(event_handler);
}

struct WindowEventHandler {
    task_manager: TaskManager,
}

impl WindowEventHandler {
    fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

impl window::EventHandler for WindowEventHandler {
    fn on_start(&mut self) {
        log::info!("main loop has been started");

        // TODO: verify GPU and Window contexts exist
        self.task_manager.run();
    }

    fn on_input(&mut self, event: window::Event) {
        // log::info!("EVENT {:?}", event);
        self.task_manager.provide(event);
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        log::info!(
            "provide new size: {}x{} {:?}",
            width,
            height,
            std::any::TypeId::of::<window::ResizeRequest>()
        );
        // if let Some(gpu) = self.gpu.as_mut() {
        //     gpu.resize_surface(width, height);
        // }
        self.task_manager
            .provide(window::ResizeRequest { width, height });
    }

    fn on_close(&mut self) {}

    fn on_draw(&mut self) {
        log::info!("Wait for presenter...");
        self.task_manager
            .wait_for::<render::FramePresenter>()
            .present();
        log::info!("...presented");
        self.task_manager.run();
    }
}
