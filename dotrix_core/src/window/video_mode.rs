//! Structure for video mode description.
use dotrix_math::Vec2u;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
/// Information about a video mode.
pub struct VideoMode {
    /// Returns the bit depth of this video mode, as in how many bits you have available
    /// per color. This is generally 24 bits or 32 bits on modern systems, depending
    /// on whether the alpha channel is counted or not.
    pub color_depth: u16,
    /// The monitor number that this video mode is valid for. Each monitor has a
    /// separate set of valid video modes.
    pub monitor_number: usize,
    /// Returns the refresh rate of this video mode.
    pub refresh_rate: u16,
    /// Resolution in pixels.
    pub resolution: Vec2u,
}

impl std::fmt::Display for VideoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} x {}, {} hz, {} bit",
            self.resolution.x, self.resolution.y, self.refresh_rate, self.color_depth)
    }
}
