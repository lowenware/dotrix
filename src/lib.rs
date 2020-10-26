mod application;
mod asset;
mod ecs;
mod input;
mod systems;
mod world;
mod window;

pub use application::{ Application, Service };
pub use asset::*;
pub use input::*;
pub use ecs::{Entity, Component, System};
pub use world::World;
pub use window::{Window};

pub struct Dotrix {
    app: Application,
}

impl Dotrix {
    pub fn application(name: &'static str) -> Self {
        Self {
            app: Application::new(name),
        }
    }

    pub fn with_system(&mut self, system: System) -> &mut Self {
        self.app.add_system(system);
        self
    }

    pub fn with_service<T: Service>(&mut self, service: T) -> &mut Self
    {
        self.app.add_service(service);
        self
    }

    /// Run the application
    pub fn run(&mut self) {
        self.app.run();
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
