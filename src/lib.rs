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
//#![warn(missing_docs)]

/// Loaders for assets from resources
pub mod loaders;
pub use loaders::{Asset, Assets, ImportResource, ResourceFile};

/// Logging utilities
pub mod log;
pub use crate::log::Log;

/// Math module
pub mod math;

/// Models abstractions
pub mod models;
pub use models::{
    Animation, Armature, Color, Image, Joint, Material, Mesh, Model, RenderModels, Transform,
    VertexAttribute, VertexBitangent, VertexJoints, VertexNormal, VertexPosition, VertexTangent,
    VertexTexture, VertexWeights,
};

/// Rendering tools and routines
pub mod graphics;
pub use graphics::{DeviceType, Display, Extent2D, Format, Frame, Gpu, Semaphore};

/// Tasks and execution
pub mod tasks;
pub use tasks::{All, Any, Mut, Output, Ref, State, Take, Task, TaskManager};

/// Utils
pub mod utils;
pub use utils::Id;

/// World
pub mod world;
pub use world::{Camera, Entity, World};

/// Window API and input events
pub mod window;
pub use window::event::{Button as ButtonEvent, Event};
pub use window::{Input, ReadInput, Window};

//pub use utils::{ Id };

// pub use dotrix_core::{All, Any, Extension, Manager, Mut, Output, Ref, State, Take, Task};
// pub use dotrix_types::{camera, type_lock, vertex, Color, Frame, Id, Transform};

// pub use dotrix_assets as assets;
// pub use dotrix_ecs as ecs;
// pub use dotrix_gpu as gpu;
// pub use dotrix_image as image;
// pub use dotrix_input as input;
// pub use dotrix_log as log;
// pub use dotrix_math as math;
// pub use dotrix_mesh as mesh;
// pub use dotrix_shader as shader;
// pub use dotrix_window as window;

// pub use assets::Assets;
// pub use camera::Camera;
// pub use ecs::World;
// use gpu::ResizeSurface;
// pub use input::Input;
// pub use mesh::{Armature, Mesh};
// pub use shader::Shader;
// pub use vertex::{Bitangent, Normal, Position, Tangent, TexUV};

// #[cfg(feature = "pbr")]
// pub use dotrix_pbr as pbr;

// #[cfg(feature = "ui")]
// pub use dotrix_ui as ui;

/// Dotrix Settings
pub trait Application {
    /// Startup
    fn startup(self, scheduler: &tasks::Scheduler, display: &mut graphics::Display);

    /// Number of workers
    fn workers(&self) -> u32 {
        std::thread::available_parallelism()
            .map(|value| value.get() as u32)
            .unwrap_or(4)
    }

    /// FPS preference
    fn fps_request(&self) -> Option<f32> {
        None
    }

    /// Log fps within an interval
    fn log_fps_interval(&self) -> Option<std::time::Duration> {
        None
    }

    /// Application name
    fn app_name(&self) -> &str {
        "Dotrix Application"
    }

    /// Application version
    fn app_version(&self) -> u32 {
        0
    }

    /// Debug flag
    fn debug(&self) -> bool {
        true
    }

    /// Device type request
    fn device_type_request(&self) -> Option<DeviceType> {
        Some(DeviceType::Discrete)
    }

    /// Is full screen
    fn full_screen(&self) -> bool {
        false
    }

    /// Preferred resolution
    fn resolution(&self) -> Extent2D {
        Extent2D {
            width: 800,
            height: 600,
        }
    }
}

/// Application launcher
pub fn run<A: Application>(application: A) {
    window::EventLoop::new(application).run();
}
