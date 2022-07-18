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

use dotrix_gpu as gpu;
use dotrix_window as window;

pub use dotrix_core::{All, Any, Manager, Mut, Ref, State, Task, Tasks};
pub use dotrix_types::{Color, Id, IdMap, Transform};

/*
pub use dotrix_core::*;
pub use dotrix_math as math;

#[cfg(feature = "egui")]
pub use dotrix_egui as egui;

#[cfg(feature = "overlay")]
pub use dotrix_overlay as overlay;

#[cfg(feature = "pbr")]
pub use dotrix_pbr as pbr;

#[cfg(feature = "primitives")]
pub use dotrix_primitives as primitives;

#[cfg(feature = "sky")]
pub use dotrix_sky as sky;

#[cfg(feature = "terrain")]
pub use dotrix_terrain as terrain;

pub mod prelude {
    pub use crate::Dotrix;
    pub use dotrix_core::ecs::{Const, Context, Mut, System};
    pub use dotrix_core::Service;
    pub use dotrix_core::{Color, Id};
}
*/

pub struct Settings {
    pub fps: u64,
    pub workers_number: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fps: 60,
            workers_number: 8,
        }
    }
}

pub struct Application {
    settings: Settings,
    task_manager: Manager,
}

// TODO: cleanup
/*
struct DummyTask;
impl Task for DummyTask {
    type Context = ();
    type Provides = os::Done;

    fn run(&mut self, _ctx: Self::Context) -> Self::Provides {
        os::Done {}
    }
}
*/

impl window::HasWindow for Application {
    fn fps(&self) -> u64 {
        self.settings.fps
    }

    fn init(&mut self, handle: window::Handle) {
        let renderer = gpu::Renderer::new(&handle, gpu::RendererOptions::default());
        renderer.clear();
        let window = window::Window::new(handle);
        self.task_manager.store(window);
        self.task_manager.store(renderer);
        // TODO: cleanup
        // self.task_manager.add(DummyTask {});
        self.task_manager.run();
    }

    fn close_request(&self) -> bool {
        false
    }

    fn on_input(&mut self /* input_event */) {}

    fn on_resize(&mut self, _width: u32, _height: u32) {}

    fn on_close(&mut self) {}

    fn on_draw(&mut self) {
        self.task_manager.wait();
        println!("Draw");
        self.task_manager.run();
    }
}

impl Application {
    pub fn new(settings: Settings) -> Self {
        let workers_number = settings.workers_number;
        Self {
            settings,
            task_manager: Manager::new(workers_number),
        }
    }

    pub fn run(self) {
        use window::HasWindow;

        self.run_window();
    }
}

pub fn application(settings: Settings) -> Application {
    Application::new(settings)
}
