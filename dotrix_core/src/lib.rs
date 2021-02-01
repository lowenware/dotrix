mod application;
pub mod animation;
pub mod assets;
mod camera;
pub mod ecs;
mod frame;
pub mod input;
pub mod renderer;
mod scheduler;
mod world;

pub use application::{ Application, Service };

pub mod components {
    pub use crate::{
        animation::Animator,
        renderer::{
            AmbientLight,
            DirLight,
            Model,
            PointLight,
            SimpleLight,
            SkyBox,
            SpotLight,
			WireFrame,
        },
    };
}

pub mod services {
    pub use crate::{
        assets::Assets,
        camera::Camera,
        input::Input,
        frame::Frame,
        input::Ray,
        renderer::Renderer,
        world::World,
    };
}

pub mod systems {
    pub use crate::{
        renderer::{
            world_renderer,
        },
        renderer::overlay_update,
        animation::skeletal_animation,
        camera::camera_control,
    };
}

use ecs::System;

pub struct Dotrix {
    app: Option<Application>,
}

#[derive(Default)]
pub struct Display {
    pub clear_color: [f64; 4],
    pub fullscreen: bool,
}

impl Dotrix {
    pub fn application(name: &'static str) -> Self {
        Self {
            app: Some(Application::new(name)),
        }
    }

    pub fn with_display(&mut self, display: Display) -> &mut Self {
        let app = self.app.as_mut().unwrap();
        app.set_display(display.clear_color, display.fullscreen);
        self
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
