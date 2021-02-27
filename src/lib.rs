//! Dotrix is a 3D game engine following ECS programming pattern with a goal to be simple and
//! feature rich. There is a [Löwenware](https://lowenware.com) team behind the project and we
//! are working on Dotrix to power up our own game projects.
//!
//! The best place to see what can be done with Dotrix is our
//! [YouTube](https://www.youtube.com/channel/UCdriNXRizbBFQhqZefaw44A) channel.
//!
//! We are also working on the editor application](https://github.com/lowenware/dotrix-editor).
//!
//! ## Getting Started
//!
//! If you are more into a practice and looking for an example code, we've prepared a good
//! [demo application](https://github.com/lowenware/dotrix/blob/main/examples/demo/demo.rs) for you
//! to get started.
//!
//! If you prefer to work with the documentation, then the best place to start is where your
//! game should start - the [`Dotrix`] application builder.
//!
//! ```no_run
//! use dotrix::Dotrix;
//!
//! fn main() {
//!     Dotrix::application("My Game")
//!         .run();
//! }
//! ```
//!
//! It is also a tool to say the engine where should be rendered the output and what [`services`]
//! and [`systems`] has to be enabled.

#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

pub use dotrix_core::*;
pub use dotrix_math as math;

#[cfg(feature = "egui")]
pub use dotrix_egui as egui;

#[cfg(feature = "terrain")]
pub use dotrix_terrain as terrain;
