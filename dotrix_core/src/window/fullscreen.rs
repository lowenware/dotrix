//! Enum for fullscreen modes.
use super::VideoMode;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// Fullscreen modes.
pub enum Fullscreen {
    /// Borderless fullscreen.
    Borderless(usize),
    /// Exclusive (classic) fullscreen.
    Exclusive(VideoMode),
}
