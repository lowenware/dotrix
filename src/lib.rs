mod application;
pub mod assets;
mod camera;
pub mod ecs;
mod input;
mod renderer;
mod scheduler;
mod world;

pub use application::{ Application, Service };
pub use input::*; // TODO: expand

pub mod components {
    pub use crate::{
        renderer::{
            Light,
            StaticModel,
        },
    };
}

pub mod services {
    pub use crate::{
        assets::Assets,
        camera::Camera,
        renderer::Renderer,
        world::World,
    };
}

pub mod systems {
    pub use crate::{
        renderer::{
            static_renderer,
        }
    };
}

use ecs::System;

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
