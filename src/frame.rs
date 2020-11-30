use std::time::{Duration, Instant};
use log::info;

pub struct Frame {
    first: Option<Instant>,
    current: Option<Instant>,
    counter_start: Option<Instant>,
    counter: u32,
    fps: Option<u32>,
    delta: Duration,
    time: Duration,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            first: None,
            current: None,
            counter_start: None,
            counter: 0,
            fps: None,
            delta: Duration::from_secs(0),
            time: Duration::from_secs(0),
        }
    }

    pub fn next(&mut self) {
        let now = Instant::now();
        if let Some(first) = self.first {
            self.time = now - first;
        } else {
            self.first = Some(now);
        }

        if let Some(current) = self.current {
            self.delta = now - current;
        }
        self.current = Some(now);

        loop {
            if let Some(counter_start) = self.counter_start {
                if now - counter_start > Duration::from_secs(1) {
                    let fps = self.counter;
                    self.fps = Some(fps);
                    self.counter = 0;
                    info!("FPS: {}", fps);
                    self.counter_start = Some(now);
                } else {
                    break;
                }
            }
            self.counter_start = Some(now);
            break;
        }

        self.counter += 1;
    }

    pub fn time(&self) -> Duration {
        self.time
    }

    pub fn fps(&self) -> u32 {
        if let Some(fps) = self.fps { fps } else { self.counter }
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}
