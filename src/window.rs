mod input;
mod map;

pub mod event;

use crate::render::Extent2D;
use std::sync::Arc;

pub use event::Event;
pub use input::ReadInput;

/// Window resize request context
#[derive(Default, Debug)]
pub struct ResizeRequest {
    /// Requested window width
    pub width: u32,
    /// Requested window height
    pub height: u32,
}

/// Dotrix window handle
#[derive(Debug, Clone)]
pub struct Handle {
    window: Arc<winit::window::Window>,
}

unsafe impl raw_window_handle::HasRawWindowHandle for Handle {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.window.raw_window_handle()
    }
}

unsafe impl raw_window_handle::HasRawDisplayHandle for Handle {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.window.raw_display_handle()
    }
}

/// Main Loop
pub struct EventLoop {
    event_loop: winit::event_loop::EventLoop<()>,
    frame_duration: std::time::Duration,
    window_handle: Handle,
}

impl EventLoop {
    pub fn set_frame_duration(&mut self, frame_duration: std::time::Duration) {
        self.frame_duration = frame_duration;
    }

    pub fn run(self, mut handler: impl EventHandler) {
        let mut pool = futures::executor::LocalPool::new();
        let _spawner = pool.spawner();

        let event_loop = self.event_loop;
        let frame_duration = self.frame_duration;
        let window_handle = self.window_handle;

        handler.on_start();

        let mut last_frame = std::time::Instant::now();

        event_loop.run(move |event, _, control_flow| {
            if let Some(dotrix_event) = map::event(&event) {
                handler.on_input(dotrix_event);
            }
            match event {
                // window control
                winit::event::Event::MainEventsCleared => {
                    if last_frame.elapsed() >= frame_duration {
                        window_handle.window.request_redraw();
                        last_frame = std::time::Instant::now();
                    }
                    pool.run_until_stalled();
                }
                // window resize
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::Resized(size),
                    ..
                } => {
                    handler.on_resize(size.width, size.height);
                }
                // window close
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::CloseRequested,
                    ..
                } => {
                    handler.on_close();
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                // draw request
                winit::event::Event::RedrawRequested(_) => {
                    handler.on_draw();
                }
                _ => {}
            }
        });
    }
}

/// Window Service
pub struct Window {
    handle: Handle,
}

impl Window {
    pub fn new(title: &str, resolution: Extent2D, _fullscreen: bool) -> (Window, EventLoop) {
        let event_loop = winit::event_loop::EventLoop::new();

        let winit_window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                resolution.width,
                resolution.height,
            ))
            // TODO: fullscreen
            .build(&event_loop)
            .expect("Failed to create a window");

        let handle = Handle {
            window: Arc::new(winit_window),
        };

        let main_loop = EventLoop {
            event_loop,
            frame_duration: std::time::Duration::from_secs_f32(0.0),
            window_handle: handle.clone(),
        };

        let window = Self { handle };

        (window, main_loop)
    }

    /// Set window title
    pub fn set_title(&self, title: &str) {
        self.handle.window.set_title(title);
    }

    /// Returns window's handle
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Returns window's resolution
    pub fn resolution(&self) -> Extent2D {
        let size = self.handle.window.inner_size();
        Extent2D {
            width: size.width,
            height: size.height,
        }
    }
}

/// Trait representing ability of application to have a window
pub trait EventHandler: Sized + 'static {
    /// Called after the `run` method called, providing correct window's handle
    fn on_start(&mut self);

    /// Input handler
    fn on_input(&mut self, event: Event);

    /// Window resize callback
    fn on_resize(&mut self, width: u32, height: u32);

    /// Window close callback
    fn on_close(&mut self);

    /// Window draw callback
    fn on_draw(&mut self);
}
