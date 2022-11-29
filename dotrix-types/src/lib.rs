//! Dotrix core crate crate provides generic features.

#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

// mod application;
mod color;
// mod cubemap;
// mod globals;
// mod pose;
// mod state;
// mod world;

// pub mod animation;
// pub mod assets;
pub mod camera;
// pub mod ecs;
// pub mod frame;
pub mod id;
pub mod type_lock;
// pub mod input;
// pub mod ray;
// pub mod renderer;
pub mod transform;
pub mod vertex;
// pub mod window;

// pub use animation::Animator;
// pub use application::{Application, IntoService, Service};
// pub use assets::Assets;
pub use camera::Camera;
pub use color::Color;
// pub use cubemap::CubeMap;
// pub use ecs::{Priority, RunLevel, System};
// pub use frame::Frame;
// pub use globals::Globals;
pub use id::Id;
// pub use input::Input;
// pub use pose::Pose;
// pub use ray::Ray;
// pub use renderer::Renderer;
// pub use state::State;
pub use transform::Transform;
pub use type_lock::TypeLock;
pub use vertex::{Bitangent, Normal, Position, Tangent, TexUV};
// pub use window::{Monitor, VideoMode, Window};
// pub use world::World;

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
