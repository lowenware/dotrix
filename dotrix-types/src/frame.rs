pub struct Frame {
    pub delta: std::time::Duration,
    pub instant: std::time::Instant,
    pub width: u32,
    pub height: u32,
    pub number: u64,
}
