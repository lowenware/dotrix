//! Dotrix core crate crate provides generic features.

#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

mod application;
mod color;
mod cubemap;
mod frame;
mod globals;
mod id;
mod pipeline;
mod pose;
mod state;
mod world;

pub mod animation;
pub mod assets;
pub mod camera;
pub mod ecs;
pub mod input;
pub mod ray;
pub mod renderer;
pub mod transform;
pub mod window;

pub use animation::Animator;
pub use application::{Application, IntoService, Service};
pub use assets::Assets;
pub use camera::Camera;
pub use color::Color;
pub use cubemap::CubeMap;
pub use ecs::{Priority, RunLevel, StateId, System};
pub use frame::Frame;
pub use globals::Globals;
pub use id::Id;
pub use input::Input;
pub use pipeline::Pipeline;
pub use pose::Pose;
pub use ray::Ray;
pub use renderer::Renderer;
pub use state::State;
pub use transform::Transform;
pub use window::{Monitor, VideoMode, Window};
pub use world::World;

#[deprecated(
    since = "0.5.0",
    note = "Please use components from dotrix crate instead"
)]
pub mod components {
    //! Dotrix core components
    pub use crate::{animation::Animator, color::Color, pose::Pose, transform::Transform};
}

#[deprecated(
    since = "0.5.0",
    note = "Please use services from dotrix crate instead"
)]
pub mod services {
    //! Services are very important part of Dotrix. Technicaly the service is a standard Rust
    //! structure with methods. Logically, services are providers of interfaces to various
    //! features.
    //!
    //! Developer should explicitly create an instance of a service and add it to a game using the
    //! [`crate::Dotrix`] application builder:
    //!
    pub use crate::{
        assets::Assets, camera::Camera, frame::Frame, globals::Globals, input::Input, ray::Ray,
        renderer::Renderer, window::Window, world::World,
    };
}

/// Application Builder
///
/// This structure is supposed to be constructed only once and usually inside of a main
/// function
///
/// You can also check full functional
/// [Dotrix Demo](https://github.com/lowenware/dotrix/blob/main/examples/demo/main.rs) example to
/// learn more about the builder.
pub struct Dotrix {
    app: Option<Application>,
}

/// Rendering output configuration
#[derive(Default)]
pub struct Display {
    /// Background clear color (RGBA)
    pub clear_color: [f64; 4],
    /// Fullscreen control (ignored in current implementation
    pub fullscreen: bool,
}

impl Dotrix {
    /// Initiates building of an application with specified name
    pub fn application(name: &'static str) -> Self {
        let mut app = Application::new(name);
        // Assets manager
        app.add_service(assets::Assets::default());
        // Camera service
        app.add_service(camera::Camera::default());
        // FPS and delta time counter
        app.add_service(frame::Frame::default());
        // Input manager
        app.add_service(input::Input::default());
        // Global buffers
        app.add_service(globals::Globals::default());
        // Render manager
        app.add_service(renderer::Renderer::default());
        // States stack
        app.add_service(state::State::default());

        // Window manager
        app.add_service(window::Window::default());
        // World manager
        app.add_service(world::World::default());

        // Renderer startup
        app.add_system(System::from(renderer::startup));
        app.add_system(System::from(camera::startup));

        // Handle resize event
        app.add_system(System::from(renderer::resize));
        app.add_system(System::from(camera::resize));
        // Bind to a new frame
        app.add_system(System::from(renderer::bind));
        // Recalculate FPS and delta time
        app.add_system(System::from(frame::bind));
        // load proj_view matrices
        app.add_system(System::from(camera::bind));

        // Calculate skeletal animations
        app.add_system(System::from(animation::skeletal));

        // Finalize frame by Renderer
        app.add_system(System::from(renderer::release));
        // Reset input events
        app.add_system(System::from(input::release));

        // Reload and clean up assets
        app.add_system(System::from(assets::assets_reload));

        Self { app: Some(app) }
    }

    /// Initiates building of an application with specified name
    pub fn bare(name: &'static str) -> Self {
        Self {
            app: Some(Application::new(name)),
        }
    }

    /// Adds system, service or extension to the application
    pub fn with<T>(&mut self, engine_unit: T) -> &mut Self
    where
        Self: ExtendWith<T>,
    {
        self.extend_with(engine_unit);
        self
    }

    /// Adds a system to the application
    pub fn with_system(&mut self, system: System) -> &mut Self {
        self.app.as_mut().unwrap().add_system(system);
        self
    }

    /// Adds a service to the application

    pub fn with_service<T: IntoService>(&mut self, service: T) -> &mut Self {
        self.app.as_mut().unwrap().add_service(service);
        self
    }

    /// Runs the application
    pub fn run(&mut self) {
        let app = self.app.take().unwrap();
        app.run();
    }
}

/// Trait providing extendablity
pub trait ExtendWith<T> {
    /// Extends self using the `extension` function
    fn extend_with(&mut self, extension: T);
}

impl ExtendWith<System> for Dotrix {
    fn extend_with(&mut self, extension: System) {
        self.app.as_mut().unwrap().add_system(extension);
    }
}

impl<T: IntoService> ExtendWith<Service<T>> for Dotrix {
    fn extend_with(&mut self, extension: Service<T>) {
        self.app.as_mut().unwrap().add_service(extension.node);
    }
}

impl<T: FnOnce(&mut Application)> ExtendWith<T> for Dotrix {
    fn extend_with(&mut self, extension: T) {
        extension(self.app.as_mut().unwrap())
    }
}

/// Count parameters
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt, $($xs:tt)* ) => (1usize + count!($($xs)*));
}

/// Recursive macro treating arguments as a progression
///
/// Expansion of recursive!(macro, A, B, C) is equivalent to the expansion of sequence
/// macro!(A)
/// macro!(A, B)
/// macro!(A, B, C)
#[macro_export]
macro_rules! recursive {
    ($macro: ident, $args: ident) => {
        $macro!{$args}
    };
    ($macro: ident, $first: ident, $($rest: ident),*) => {
        $macro!{$first, $($rest),*}
        recursive!{$macro, $($rest),*}
    };
}
