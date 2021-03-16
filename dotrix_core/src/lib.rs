//! Dotrix core crate crate provides generic features.

#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

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
//! Component usually is just a data structure with or without associated methods. In ECS
//! pattern components represent properties of entities: velocity, weight, model, rigid body, color
//! etc.
//!
//! Set of components in the entity defines an Archetype. Entities of the same
//! Archetype are being grouped together in the [`crate::services::World`] storage, that
//! makes search fast and linear.
//!
//! When planning the architecture of your game, developer should think about components not only
//! as of properties, but also as of search tags. For example, if you are making physics for your
//! game, and you have a `Car` and a `SpringBoard`, you may want to have the same named components,
//! so you can easily query all `Cars` or all `SpringBoards`. But as soon as you will need to
//! calculate physics for entities of the both types, you should add some component like 
//! `RigidBody` to the entities, so you can calculate physics for all entities who have that
//! component.
//!
//! ## Usefull references
//! - To learn how to spawn and query entities, continue reading with [`crate::services::World`]
//! - To learn how to implement systems [`crate::systems`]
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
//! Services are very important part of Dotrix. Technicaly the service is a standard Rust
//! structure with methods. Logically, services are providers of interfaces to various
//! features.
//!
//! Developer should explicitly create an instance of a service and add it to a game using the
//! [`crate::Dotrix`] application builder:
//!
//! ```no_run
//! use dotrix_core::{
//!     Dotrix,
//!     services::{ World },
//! };
//!
//! // in fn main()
//! Dotrix::application("My Game")
//!     .with_service(World::default())
//!     .run();
//!
//! ```
//! After adding a service to your game you can access it inside of [`crate::systems`].
//!

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
//! Systems implement logic of your game. A system take a set of [`crate::services`] as
//! parameters and implements a feature. For example, here is a system that moves camera by X
//! axis if Right mouse button is pressed:
//!
//! ```no_run
//! use dotrix_core::{
//!     Dotrix,
//!     input::{ ActionMapper, Button, State as InputState, Mapper }, 
//!     ecs::{ Const, Mut, System },
//!     services::{ Camera, Frame, Input }, 
//! };
//!
//! // camera moving system
//! pub fn camera_move_x(mut camera: Mut<Camera>, input: Const<Input>, frame: Const<Frame>) {
//!     let time_delta = frame.delta().as_secs_f32();
//!
//!     if input.button_state(Button::MouseRight) == Some(InputState::Hold) {
//!         camera.target.x += 1.0 * time_delta;
//!     }
//! }
//!
//! fn main() {
//!     Dotrix::application("My Game")
//!         // add system to yout application
//!         .with_system(System::from(camera_move_x))
//!         // services should also be there
//!         .with_service(Camera::default())
//!         .with_service(Frame::default())
//!         .with_service(Input::new(Box::new(Mapper::<Action>::new())))
//!         .run();
//! }
//!
//! // Mapping is required for the Input service
//! #[derive(PartialEq, Eq, Clone, Copy, Hash)]
//! enum Action {}
//!
//! impl ActionMapper<Action> for Input {
//!     fn action_mapped(&self, action: Action) -> Option<&Button> {
//!         let mapper = self.mapper::<Mapper<Action>>();
//!         mapper.get_button(action)
//!     }
//! }
//! ```
//!
//! [`crate::ecs::Mut`] and [`crate::ecs::Const`] are just the accessors for services to keep
//! Rust mutability controls. The set of services that system takes as arguments is up to
//! developer. It can be effortlessly extended at any time. The order of [`crate::services`] in
//! arguments list does not matter.
//!
//! What is also important to mention here, is that all services used in systems must be added
//! to your game using the application builder.
//!
//! ## Systems with context
//!
//! Sometimes it is usefull to preserve some data between system executions. It can be done using
//! a [`crate::Service`], but it has sense only when the data has to be shared between different
//! systems. Otherwise it is better to use the [`crate::ecs::Context`].
//!
//! System can have only one context and if it does, the context must be always passed as a first
//! argument. Another requirement is that [`crate::ecs::Context`] structure must implement the
//! `Default` trait.
//!
//! ## Example
//! ```no_run
//! use dotrix_core::{
//!     ecs::Context,
//! };
//!
//! #[derive(Default)]
//! struct Counter {
//!     value: usize,
//! }
//!
//! fn count_up(mut counter: Context<Counter>) {
//!     counter.value += 1;
//!     println!("The `counter_up` system has been called {} times", counter.value);
//! }
//! ```
//!
//! ## System run levels
//!
//! Developer can affect the execution of systems by assigning them specific run levels.
//! If [`crate::ecs::RunLevel`] is not set explicitly, the `RunLevel::Standard` will be used.
//!
//! ```no_run
//! use dotrix_core::{
//!     Dotrix,
//!     services::Assets,
//!     ecs::{ Mut, System, RunLevel },
//! };
//!
//! fn init_game(mut assets: Mut<Assets>) {
//!     println!("Starting my super game");
//!     assets.import("/path/to/my/asset.png");
//! }
//!
//! fn main() {
//!     Dotrix::application("My Game")
//!         .with_system(System::from(init_game).with(RunLevel::Startup))
//!         .with_service(Assets::new())
//!         .run();
//! }
//! ```

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

/// Application Builder
///
/// This structure is supposed to be constructed only once and usually inside of a main function
///
/// ## Example
///
/// ```no_run
/// use dotrix_core::{
///     Dotrix,
///     ecs::{ Mut, System, RunLevel },
///     services::{ Assets, Camera, World },
///     systems::{ world_renderer },
/// };
///
/// use dotrix_math::Point3;
///
/// fn main() {
///     Dotrix::application("My Game")
///         .with_system(System::from(startup).with(RunLevel::Startup))
///         .with_system(System::from(world_renderer).with(RunLevel::Render))
///         .with_service(Assets::new())
///         .with_service(Camera {
///             y_angle: 0.0,
///             xz_angle: 0.0,
///             target: Point3::new(0.0, 10.0, 0.0),
///             distance: 5.0,
///             ..Default::default()
///         })
///         .with_service(World::new())
///         .run();
/// }
///
/// fn startup(mut assets: Mut<Assets>) {
///     // initialize app and load assets
/// }
/// ```
///
/// You can also check full functional
/// [Dotrix Demo](https://github.com/lowenware/dotrix/blob/main/examples/demo/demo.rs) example to
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
        Self {
            app: Some(Application::new(name)),
        }
    }

    /// Configures rendering output
    pub fn with_display(&mut self, display: Display) -> &mut Self {
        let app = self.app.as_mut().unwrap();
        app.set_display(display.clear_color, display.fullscreen);
        self
    }

    /// Adds a system to the application
    pub fn with_system(&mut self, system: System) -> &mut Self {
        self.app.as_mut().unwrap().add_system(system);
        self
    }

    /// Adds a service to the application
    pub fn with_service<T: Service>(&mut self, service: T) -> &mut Self
    {
        self.app.as_mut().unwrap().add_service(service);
        self
    }

    /// Runs the application
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
