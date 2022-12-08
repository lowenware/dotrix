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
#![warn(missing_docs)]

pub use dotrix_core::{All, Any, Extension, Manager, Mut, Output, Ref, State, Take, Task, Tasks};
pub use dotrix_types::{camera, type_lock, vertex, Color, Frame, Id, Transform};

pub use dotrix_assets as assets;
pub use dotrix_ecs as ecs;
pub use dotrix_gpu as gpu;
pub use dotrix_image as image;
pub use dotrix_input as input;
pub use dotrix_log as log;
pub use dotrix_math as math;
pub use dotrix_mesh as mesh;
pub use dotrix_shader as shader;
pub use dotrix_window as window;

pub use assets::Assets;
pub use camera::Camera;
pub use ecs::World;
pub use input::Input;
pub use log::Log;
pub use mesh::{Armature, Mesh};
pub use shader::Shader;
pub use vertex::{Bitangent, Normal, Position, Tangent, TexUV};

#[cfg(feature = "pbr")]
pub use dotrix_pbr as pbr;

#[cfg(feature = "ui")]
pub use dotrix_ui as ui;
//pub use ecs::World;

/*
pub use dotrix_core::*;
pub use dotrix_math as math;

#[cfg(feature = "egui")]
pub use dotrix_egui as egui;

#[cfg(feature = "overlay")]
pub use dotrix_overlay as overlay;


#[cfg(feature = "primitives")]
pub use dotrix_primitives as primitives;

#[cfg(feature = "sky")]
pub use dotrix_sky as sky;

#[cfg(feature = "terrain")]
pub use dotrix_terrain as terrain;

pub mod prelude {
    pub use crate::Dotrix;
    pub use dotrix_core::ecs::{Const, Context, Mut, System};
    pub use dotrix_core::Service;
    pub use dotrix_core::{Color, Id};
}
*/

/// Dotrix Core data structure
pub struct Core {
    controller: Controller,
    extensions: Extensions,
}

impl Core {
    pub fn schedule<T: Task>(&mut self, task: T) {
        self.controller.manager.schedule(task);
    }
    pub fn extend_with<T: Extension>(&mut self, extension: T) {
        extension.add_to(&mut self.controller.manager);
        self.extensions
            .registry
            .insert(std::any::TypeId::of::<T>(), Box::new(extension));
    }
}

/// Dotrix Extensions registry
pub struct Extensions {
    registry: std::collections::HashMap<std::any::TypeId, Box<dyn Extension>>,
}

impl Default for Extensions {
    fn default() -> Self {
        Self {
            registry: std::collections::HashMap::new(),
        }
    }
}

/// Dotrix Core Settings
pub struct Settings {
    /// Application Window Title
    pub title: String,
    /// Desired FPS
    pub fps: f32,
    /// Number of workers
    pub workers: u32,
    /// If true, Dotrix will try to run in full screen mode
    pub full_screen: bool,
    /// If true, Dotrix will take care about screen clearing
    pub clear_screen: bool,
}

impl Settings {
    pub fn validate(&mut self) {
        if self.workers < 2 {
            self.workers = 2;
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            title: String::from("Dotrix Application"),
            fps: 60.0,
            workers: 8,
            full_screen: false,
            clear_screen: true,
        }
    }
}

/// Application Controller object
struct Controller {
    manager: Manager,
    settings: Settings,
    //debug:
    count: u32,
    instant: std::time::Instant,
}

impl window::Controller for Controller {
    fn fps(&self) -> f32 {
        self.settings.fps
    }

    fn init(&mut self, window_handle: window::Handle, width: u32, height: u32) {
        let renderer = gpu::Gpu::new(gpu::Descriptor {
            window_handle: &window_handle,
            fps_request: self.settings.fps,
            surface_size: [width, height],
            sample_count: 4, // TODO: MSAA setting
        });
        let window = window::Window::new(window_handle);
        // window.set_title(&self.settings.title);
        // window.set_full_screen(self.settings.full_screen);

        self.manager.store(window);
        self.manager.store(renderer);
        // rendering tasks
        self.manager.schedule(gpu::CreateFrame::default());
        self.manager.schedule(gpu::ClearFrame::default());
        self.manager.schedule(gpu::SubmitCommands::default());
        self.manager.schedule(gpu::PresentFrame::default());
        self.manager.schedule(gpu::ResizeSurface::default());

        self.manager.schedule(input::ListenTask::default());

        self.manager.run();
    }

    fn close_request(&self) -> bool {
        false
    }

    fn on_input(&mut self, event: input::Event) {
        // log::info!("EVENT {:?}", event);
        self.manager.provide(event);
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        log::info!("provide new size: {}x{}", width, height);
        self.manager.provide(gpu::SurfaceSize { width, height });
    }

    fn on_close(&mut self) {}

    fn on_draw(&mut self) {
        self.count += 1;
        self.manager.wait_for::<gpu::PresentFrame>();
        self.manager.run();
        if self.instant.elapsed().as_secs_f32() >= 1.0 {
            log::info!("real fps: {}", self.count);
            self.instant = std::time::Instant::now();
            self.count = 0;
        }
    }
}

impl Controller {
    fn run(self) {
        use window::Controller;
        self.run_window();
    }
}

/// Dotrix interface for applications
pub trait Application: 'static + Send {
    /// Provides a possibility for the Application to change Dotrix Settings
    fn configure(&self, _settings: &mut Settings) {}
    /// Allows application to initialize all necessary context and tasks
    fn init(&self, _manager: &mut Manager) {}
}

impl<T: Application> From<T> for Core {
    fn from(app: T) -> Self {
        let mut settings = Settings::default();
        app.configure(&mut settings);
        settings.validate();

        let mut manager = Manager::new(settings.workers);
        let tasks = manager.context();

        let world = World::new();

        app.init(&mut manager);

        manager.store(app);
        manager.store(tasks);
        manager.store(world);

        Core {
            controller: Controller {
                manager,
                settings,
                count: 0,
                instant: std::time::Instant::now(),
            },
            extensions: Extensions::default(),
        }
    }
}

/// Runs application
pub fn run<A: Application, F>(app: A, setup_extensions: F)
where
    F: FnOnce(&mut Core),
{
    let mut core = Core::from(app);
    setup_extensions(&mut core);

    let Core {
        controller,
        extensions,
    } = core;
    controller.manager.store(extensions);

    controller.run();
}
