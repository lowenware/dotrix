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

pub mod extensions;
pub mod settings;

pub use dotrix_core::{All, Any, Extension, Manager, Mut, Output, Ref, State, Take, Task};
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
use gpu::ResizeSurface;
pub use input::Input;
pub use log::Log;
pub use mesh::{Armature, Mesh};
pub use shader::Shader;
pub use vertex::{Bitangent, Normal, Position, Tangent, TexUV};

#[cfg(feature = "pbr")]
pub use dotrix_pbr as pbr;

#[cfg(feature = "ui")]
pub use dotrix_ui as ui;

pub use extensions::Extensions;
pub use settings::Settings;

/// Dotrix interface for applications
pub trait Application: 'static + Send {
    /// Provides a possibility for the Application to change Dotrix Settings
    fn configure(&self, _settings: &mut Settings) {}
    /// Allows application to initialize all necessary context and tasks
    fn init(&self, _builder: &Manager) {}
}

/// Runs application
pub fn run<A: Application, F>(app: A, setup_extensions: F)
where
    F: FnOnce(&mut extensions::Loader),
{
    let mut settings = Settings::default();
    let mut extensions = Extensions::default();

    // configure dotrix for the app
    app.configure(&mut settings);

    // Set target output, so scheduler can build the dependency graph
    let manager = Manager::new::<gpu::FrameOutput>(settings.workers);
    // load extensions
    {
        let mut loader = extensions::Loader::new(&manager, &mut extensions);

        setup_extensions(&mut loader);
    }
    manager.store(extensions);

    // initialize application
    app.init(&manager);
    manager.store(app);

    Dotrix::new(settings, manager).run();
}

/// Application Controller object
pub struct Dotrix {
    manager: Manager,
    settings: Settings,
    frames: u64,
}

impl Dotrix {
    pub fn new(settings: Settings, manager: Manager) -> Self {
        Self {
            manager,
            settings,
            frames: 0,
        }
    }

    pub fn run(self) {
        use window::Controller;
        let fps_limit = self.settings.fps_limit.clone();
        self.run_window(fps_limit);
    }
}

impl window::Controller for Dotrix {
    fn init(&mut self, window_handle: window::Handle, width: u32, height: u32) {
        let gpu = gpu::Gpu::new(gpu::Descriptor {
            window_handle: &window_handle,
            fps_limit: self.settings.fps_limit,
            surface_size: [width, height],
            sample_count: 4, // TODO: MSAA setting
        });
        let window = window::Window::new(window_handle);
        // TODO:
        // window.set_title(&self.settings.title);
        // window.set_full_screen(self.settings.full_screen);
        let world = World::new();

        // add last globals
        self.manager.store(gpu);
        self.manager.store(window);
        self.manager.store(world);

        // schedule rendering tasks
        self.manager.schedule(gpu::CreateFrame::default());
        self.manager.schedule(gpu::ClearFrame::default());
        self.manager.schedule(gpu::SubmitCommands::default());
        self.manager.schedule(gpu::ResizeSurface::default());

        // Input listening
        self.manager.schedule(input::ListenTask::default());

        // register data provided by window controller
        self.manager.register::<gpu::SurfaceSize>(0);
        self.manager.register::<input::Event>(0);

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
        log::info!(
            "provide new size: {}x{} {:?}",
            width,
            height,
            std::any::TypeId::of::<gpu::SurfaceSize>()
        );
        self.manager.provide(gpu::SurfaceSize { width, height });
    }

    fn on_close(&mut self) {}

    fn on_draw(&mut self) {
        log::debug!("Draw Request {}", self.frames);
        self.manager.wait_for::<gpu::FrameOutput>().present();

        self.manager.run();
        self.frames += 1;
    }
}
