// mod color;
// mod frame;
// mod cubemap;
// mod globals;
// mod pose;
// mod state;
// mod world;

// pub mod assets;
// pub mod camera;
// pub mod ecs;
// pub mod frame;
/// Buffer layout
pub mod buffer_layout;
pub use buffer_layout::{BufferLayout, LayoutInBuffer, MeshLayout, MeshVerticesLayout};
/// Typed ids
pub mod id;
pub use id::Id;
/// Typed safety lock
pub mod type_lock;
pub use type_lock::{Lock, LockMode, TypeLock};
// pub mod input;
// pub mod ray;
// pub mod renderer;
//pub mod transform;
//pub mod vertex;
// pub mod window;

// pub use application::{Application, IntoService, Service};
// pub use assets::Assets;
//pub use camera::Camera;
//pub use color::Color;
//pub use frame::Frame;
// pub use cubemap::CubeMap;
// pub use ecs::{Priority, RunLevel, System};
// pub use frame::Frame;
// pub use globals::Globals;
// pub use input::Input;
// pub use pose::Pose;
// pub use ray::Ray;
// pub use renderer::Renderer;
// pub use state::State;
//pub use transform::Transform;
// pub use vertex::{Bitangent, Normal, Position, Tangent, TexUV};
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