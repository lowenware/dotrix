use crate::gpu;

/// Dotrix Settings
pub struct Settings {
    /// Application name
    pub application_name: String,
    /// Application version
    pub application_version: u32,
    /// Limit FPS to some value or set to None for Max
    pub fps_limit: Option<f32>,
    /// Log fps within an interval
    pub log_fps_interval: Option<std::time::Duration>,
    /// Number of workers
    pub workers: u32,
    /// If true, Dotrix will try to run in full screen mode
    pub full_screen: bool,
    /// Enable or disable debug outputs
    pub debug: bool,
    /// GPU selection preference
    pub gpu_type_request: Option<gpu::DeviceType>,
}

impl Settings {
    /// Sets application name
    pub fn application_name(mut self, value: impl Into<String>) -> Self {
        self.application_name = value.into();
        self
    }

    /// Sets application version
    pub fn application_version(mut self, value: u32) -> Self {
        self.application_version = value;
        self
    }

    /// Sets FPS limit preference
    pub fn fps_limit(mut self, value: Option<f32>) -> Self {
        self.fps_limit = value;
        self
    }

    /// Sets FPS logging
    pub fn log_fps_interval(mut self, value: Option<std::time::Duration>) -> Self {
        self.log_fps_interval = value;
        self
    }

    /// Sets number of workers
    pub fn workers(mut self, value: u32) -> Self {
        self.workers = value;
        self
    }

    /// Sets full-screen mode
    pub fn full_screen(mut self, value: bool) -> Self {
        self.full_screen = value;
        self
    }

    /// Sets debug mode
    pub fn debug(mut self, value: bool) -> Self {
        self.debug = value;
        self
    }

    /// Sets GPU type preference
    pub fn gpu_type_request(mut self, value: Option<gpu::DeviceType>) -> Self {
        self.gpu_type_request = value;
        self
    }

    /// Validates and corrects settings values
    pub fn validate(&mut self) {
        if self.workers < 2 {
            self.workers = 2;
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            application_name: String::from("Dotrix"),
            application_version: 0,
            fps_limit: None,
            log_fps_interval: None,
            workers: 8,
            full_screen: false,
            debug: true,
            gpu_type_request: None,
        }
    }
}
