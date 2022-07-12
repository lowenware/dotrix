use std::sync::Arc;

const MICROSECONDS_PER_SECOND: u64 = 1_000_000;

/// Dotrix window handle
pub struct Handle {
    window: Arc<winit::window::Window>,
}

unsafe impl raw_window_handle::HasRawWindowHandle for Handle {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.window.raw_window_handle()
    }
}

/// Window Service
pub struct Window {
    handle: Handle,
}

impl Window {
    pub fn new(handle: Handle) -> Self {
        Self {
            handle,
        }
    }
}

/// Trait representing ability of application to have a window
pub trait HasWindow: Sized + 'static {

    fn fps(&self) -> u64;

    fn init(&mut self, handle: Handle);

    fn close_request(&self) -> bool;

    fn on_input(&mut self /* input_event */);

    fn on_resize(&mut self, _width: u32, _height: u32);

    fn on_close(&mut self);

    fn on_draw(&mut self);

    fn run_window(mut self) {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = Arc::new(
            winit::window::Window::new(&event_loop)
                .expect("Window must be created")
        );
        let fps = self.fps();
        let frame_duration = std::time::Duration::from_micros(MICROSECONDS_PER_SECOND / fps);

        let mut pool = futures::executor::LocalPool::new();
        let _spawner = pool.spawner();

        self.init(Handle {
            window: Arc::clone(&window)
        });

        let mut last_frame = std::time::Instant::now();

        event_loop.run(move |event, _, control_flow| {
            match event {
                // window control
                winit::event::Event::MainEventsCleared => {
                    if last_frame.elapsed() >= frame_duration {
                        if self.close_request() {
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        } else {
                            window.request_redraw();
                        }
                        last_frame = std::time::Instant::now();
                    }
                    pool.run_until_stalled();
                }
                // window resize
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::Resized(size), ..
                } => {
                    self.on_resize(size.width, size.height);
                }
                // window close
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::CloseRequested,
                    ..
                } => {
                    self.on_close();
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                // draw request
                winit::event::Event::RedrawRequested(_) => {
                    self.on_draw();
                }
                _ => {}
            }
        });
    }
}
