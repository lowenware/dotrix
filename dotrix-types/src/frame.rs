pub struct Frame {
    pub fps: f32,
    pub delta: std::time::Duration,
    pub instant: std::time::Instant,
    pub width: u32,
    pub height: u32,
    pub number: u64,
    pub scale_factor: f32,
    pub resized: bool,
}

impl Frame {
    pub fn delta(&self) -> std::time::Duration {
        self.delta
    }
}
