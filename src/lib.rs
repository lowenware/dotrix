mod application;
mod assets;
mod camera;
mod ecs;
mod input;
mod renderer;
mod scheduler;
mod services;
mod world;

pub use application::{ Application };
pub use assets::{ Assets, Texture, Mesh };
pub use camera::Camera;
pub use ecs::{ Entity, Const, Component, Mut, RunLevel, System };
pub use input::*; // TODO: expand
pub use renderer::{ Light, Renderer, StaticModel, static_renderer };
pub use services::Service;
pub use world::World;

pub struct Dotrix {
    app: Option<Application>,
}

impl Dotrix {
    pub fn application(name: &'static str) -> Self {
        Self {
            app: Some(Application::new(name)),
        }
    }

    pub fn with_system(&mut self, system: System) -> &mut Self {
        self.app.as_mut().unwrap().add_system(system);
        self
    }

    pub fn with_service<T: Service>(&mut self, service: T) -> &mut Self
    {
        self.app.as_mut().unwrap().add_service(service);
        self
    }

    /// Run the application
    pub fn run(&mut self) {
        let app = self.app.take().unwrap();
        app.run();
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
