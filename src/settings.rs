/// Dotrix Core Settings
pub struct Settings {
    /// Application Window Title
    pub title: String,
    /// Limit FPS to some value or set to None for Max
    pub fps_limit: Option<f32>,
    /// Number of workers
    pub workers: u32,
    /// If true, Dotrix will try to run in full screen mode
    pub full_screen: bool,
    /// If true, Dotrix will take care about screen clearing
    pub clear_screen: bool,
}

impl Settings {
    pub fn validate(&mut self) {
        if self.workers < 2 {
            self.workers = 2;
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            title: String::from("Dotrix Application"),
            fps_limit: None,
            workers: 8,
            full_screen: false,
            clear_screen: true,
        }
    }
}
