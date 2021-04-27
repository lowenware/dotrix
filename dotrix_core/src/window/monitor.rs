//! Structure for monitor description.
use dotrix_math::Vec2u;
use super::video_mode::VideoMode;

#[derive(Debug, PartialEq, Clone)]
/// Information about a monitor.
pub struct Monitor {
    /// Human-readable (kind of) name of the monitor.
    pub name: String,
    /// Internal monitor number.
    pub number: usize,
    /// The scale factor that can be used to map logical pixels to physical pixels,
    /// and vice versa.
    pub scale_factor: f32,
    /// Maximum resolution in pixels.
    pub size: Vec2u,
    /// All video modes supported by the monitor.
    pub video_modes: Vec<VideoMode>,
}
