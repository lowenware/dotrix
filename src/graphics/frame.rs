use super::{Display, Extent2D, FramePresenter};
use crate::log;
use crate::tasks::{All, Any, Mut, OutputChannel, Ref, Task};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Frame Data
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    /// Current FPS
    pub fps: f32,
    /// Time passed since last frame
    pub delta: std::time::Duration,
    /// Timestamp of the frame
    pub timestamp: std::time::Instant,
    /// Frame resolution
    pub resolution: Extent2D,
    /// Absolute frame number
    pub number: u64,
    /// Frame's swapchain index
    pub swapchain_index: u32,
    /// Frame scale factor
    pub scale_factor: f32,
    /// Shows if frame was resized
    pub resized: bool,
}

/// Task, responsible for frame creation
pub struct CreateFrame {
    frame_counter: u64,
    fps_request: Option<f32>,
    log_fps_interval: Option<Duration>,
    log_fps_timestamp: Instant,
    last_frame: Option<Instant>,
    frames_duration: VecDeque<Duration>,
}

impl Default for CreateFrame {
    fn default() -> Self {
        Self {
            frame_counter: 0,
            fps_request: None,
            log_fps_interval: None,
            log_fps_timestamp: Instant::now(),
            last_frame: None,
            frames_duration: VecDeque::with_capacity(60),
        }
    }
}

impl CreateFrame {
    /// Sets interval for FPS logging, if None, it won't be logged
    pub fn log_fps_interval(mut self, interval: Option<Duration>) -> Self {
        self.log_fps_interval = interval;
        self
    }

    /// Sets FPS limit
    pub fn fps_request(mut self, fps_request: Option<f32>) -> Self {
        self.fps_request = fps_request;
        self
    }
}

impl Task for CreateFrame {
    type Context = (Mut<Display>,);
    type Output = Frame;

    fn run(&mut self, (mut display,): Self::Context) -> Self::Output {
        log::debug!("CreateFrame::run() -> begin");
        let frame_number = self.frame_counter + 1;
        let now = Instant::now();
        let delta = self
            .last_frame
            .replace(now)
            .map(|i| i.elapsed())
            .unwrap_or_else(|| Duration::from_secs_f32(1.0 / self.fps_request.unwrap_or(60.0)));

        log::debug!("CreateFrame::run() -> delta: {:?}", delta);
        // resize surface before aquiring of the new frame
        let mut resized = false;
        log::debug!("CreateFrame::run() -> display.surface_resize_request()");
        // NOTE: on iOS winit.inner_size() is not possible to use in a thread. We need a different
        // surface size / resize solution
        // if display.surface_resize_request() {
        if self.last_frame.is_none() {
            log::debug!("surface resized");
            display.resize_surface();
            resized = true;
        }
        log::debug!("CreateFrame::run() -> display.surface_resolution()");
        let surface_resolution = display.surface_resolution();

        log::debug!("CreateFrame::run() -> begin display.next_frame()");
        let swapchain_index = display.next_frame();
        log::debug!("CreateFrame::run() -> end display.next_frame()");
        // TODO: scale factor comes from a window, so shall it be taken from `Window` instance?
        let scale_factor = 1.0;

        // update frames durations buffer
        if self.frames_duration.len() == self.frames_duration.capacity() {
            self.frames_duration.pop_back();
        }

        self.frames_duration.push_front(delta);

        // calculate fps
        let frames = self.frames_duration.len() as f32;
        let duration: f32 = self.frames_duration.iter().map(|d| d.as_secs_f32()).sum();
        let fps = frames / duration;

        self.frame_counter = frame_number;

        if let Some(log_fps_interval) = self.log_fps_interval.as_ref().cloned() {
            if self.log_fps_timestamp.elapsed() > log_fps_interval {
                log::info!("FPS: {:.02}", fps);
                self.log_fps_timestamp = Instant::now();
            }
        }

        let frame = Frame {
            fps,
            delta,
            timestamp: now,
            resolution: surface_resolution,
            number: frame_number,
            swapchain_index,
            scale_factor,
            resized,
        };
        log::debug!("CreateFrame::run() -> {:?}", frame);
        frame
    }
}

/// Task, responsible for frame submition to be presented
#[derive(Default)]
pub struct SubmitFrame {}

impl Task for SubmitFrame {
    type Context = (Ref<Display>, Any<Frame>, All<RenderPass>);
    type Output = FramePresenter;

    fn output_channel(&self) -> OutputChannel {
        OutputChannel::Scheduler
    }

    fn run(&mut self, (display, frame, _): Self::Context) -> Self::Output {
        log::info!("get presenter");
        display.presenter(frame.swapchain_index)
    }
}

/// Render Pass Output
pub struct RenderPass {}
