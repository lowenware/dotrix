mod input;

pub mod event;

use std::sync::Arc;
use std::time;

pub use event::Event;
pub use input::Input;
pub use input::ReadInput;
use winit::event::StartCause;

use crate::graphics::{self, Display, DisplaySetup, Extent2D};
use crate::tasks::TaskManager;
use crate::Application;

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
pub struct Instance {
    winit_window: Arc<winit::window::Window>,
}

impl Instance {
    pub fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::HasWindowHandle;
        self.winit_window.window_handle()
    }

    pub fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        use raw_window_handle::HasDisplayHandle;
        self.winit_window.display_handle()
    }

    pub fn resolution(&self) -> Extent2D {
        let size = self.winit_window.inner_size();
        Extent2D {
            width: size.width,
            height: size.height,
        }
    }
}

/// Main Loop
pub struct EventLoop<T: Application> {
    application: Option<T>,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    frame_duration: std::time::Duration,
    window_instance: Option<Instance>,
    task_manager: TaskManager,
}

impl<T: Application> EventLoop<T> {
    pub fn new(application: T) -> Self {
        let workers = application.workers();
        let frame_duration = std::time::Duration::from_secs_f32(
            application
                .fps_request()
                .as_ref()
                .map(|fps_request| 1.0 / fps_request)
                .unwrap_or(0.0),
        );
        let task_manager = TaskManager::new::<graphics::FramePresenter>(workers);
        task_manager.register::<Event>(0);

        Self {
            application: Some(application),
            frame_duration,
            request_redraw: false,
            wait_cancelled: false,
            close_requested: false,
            window_instance: None,
            task_manager,
        }
    }

    // pub fn set_frame_duration(&mut self, frame_duration: std::time::Duration) {
    //     self.frame_duration = frame_duration;
    // }
    pub fn run(&mut self) {
        let event_loop =
            winit::event_loop::EventLoop::new().expect("Could not create window event loop");

        event_loop.run_app(self).ok();
    }
}

impl<T: Application> winit::application::ApplicationHandler for EventLoop<T> {
    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        // log::info!("new_events: {cause:?}");
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. });
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("resumed");
        if self.window_instance.is_some() {
            panic!("Window suspend/resume is not supported yet");
        }
        let app = self
            .application
            .take()
            .expect("Application instance was not provided");

        let resolution = app.resolution();

        let window_attributes = winit::window::Window::default_attributes()
            .with_title(app.app_name())
            .with_inner_size(winit::dpi::LogicalSize::new(
                resolution.width,
                resolution.height,
            ))
            .with_fullscreen(if app.full_screen() {
                Some(winit::window::Fullscreen::Borderless(None))
            } else {
                None
            });

        let window_instance = Instance {
            winit_window: Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Could not create window"),
            ),
        };

        // log::debug!("Current monitor: {:?}", window_instance.winit_window.current_monitor());
        // log::debug!("Primary monitor: {:?}", window_instance.winit_window.primary_monitor());
        //
        // for monitor in window_instance.winit_window.available_monitors() {
        //    for mode in monitor.video_modes() {
        //        log::debug!("Monitor: {:?} ({:?}) -> {:?}", monitor.name(), monitor.size(), mode.size());
        //    }
        // }

        let display_setup = DisplaySetup {
            window_instance: window_instance.clone(),
            app_name: app.app_name(),
            app_version: app.app_version(),
            debug: app.debug(),
            device_type_request: app.device_type_request(),
        };

        let mut display = Display::new(display_setup);
        {
            let scheduler = self.task_manager.scheduler();

            let create_frame_task = graphics::CreateFrame::default()
                .log_fps_interval(app.log_fps_interval())
                .fps_request(app.fps_request());
            scheduler.add_task(create_frame_task);

            let submit_frame_task = graphics::SubmitFrame::default();
            scheduler.add_task(submit_frame_task);

            scheduler.add_task(input::ReadInput::default());

            app.startup(&scheduler, &mut display);

            // add Display context
            scheduler.add_context(display);
        }

        self.window_instance = Some(window_instance);

        self.task_manager.run();
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                let (horizontal, vertical) = delta;
                let input_event = event::Event::MouseMove {
                    horizontal,
                    vertical,
                };
                self.task_manager.provide(input_event);
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                let button = event::Button::from(&event);
                let input_event = match event.state {
                    winit::event::ElementState::Pressed => event::Event::ButtonPress {
                        button,
                        text: event.text.as_ref().map(|smol_str| smol_str.to_string()),
                    },
                    winit::event::ElementState::Released => event::Event::ButtonRelease { button },
                };
                self.task_manager.provide(input_event);
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                let input_event = event::Event::MouseScroll {
                    delta: match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            event::MouseScroll::Lines {
                                horizontal: x,
                                vertical: y,
                            }
                        }
                        winit::event::MouseScrollDelta::PixelDelta(position) => {
                            event::MouseScroll::Pixels {
                                horizontal: position.x,
                                vertical: position.y,
                            }
                        }
                    },
                };
                self.task_manager.provide(input_event);
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let mouse_button = event::Button::from(&button);
                let input_event = match state {
                    winit::event::ElementState::Pressed => event::Event::ButtonPress {
                        button: mouse_button,
                        text: None,
                    },
                    winit::event::ElementState::Released => event::Event::ButtonRelease {
                        button: mouse_button,
                    },
                };
                self.task_manager.provide(input_event);
            }
            winit::event::WindowEvent::RedrawRequested => {
                if let Some(instance) = self.window_instance.as_ref() {
                    instance.winit_window.pre_present_notify();
                }
                // log::debug!("Wait for presenter...");
                self.task_manager
                    .wait_for::<graphics::FramePresenter>()
                    .present();
                // log::debug!("...presented");
                self.task_manager.run();
                // Note: can be used for debug
                // fill::fill_window(window);
                // handler.on_draw();
            }
            winit::event::WindowEvent::Resized(size) => {
                log::info!("Resized: {size:?}");
                self.task_manager.provide(ResizeRequest {
                    width: size.width,
                    height: size.height,
                });
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        /* log::debug!(
            "about_to_wait(request_redraw: {}, wait_cancelled: {}, close_requested: {})",
            self.request_redraw,
            self.wait_cancelled,
            self.close_requested,
        );
        */

        // NOTE: to wait
        // event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait),

        if !self.wait_cancelled {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                time::Instant::now() + self.frame_duration,
            ));
            self.request_redraw = true;
        }

        // TODO: implement FPS control
        if self.request_redraw && !self.wait_cancelled && !self.close_requested {
            if let Some(instance) = self.window_instance.as_ref() {
                instance.winit_window.request_redraw();
            }
            self.request_redraw = false;
        }
        // NOTE: to poll
        // std::thread::sleep(POLL_SLEEP_TIME);
        // event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        if self.close_requested {
            //handler.on_close();
            event_loop.exit();
        }
    }

    /*
    pub fn run(self, mut handler: impl EventHandler) {
        let mut pool = futures::executor::LocalPool::new();
        let _spawner = pool.spawner();

        let event_loop = self.event_loop;
        let frame_duration = self.frame_duration;
        let window_handle = self.window_handle;

        handler.on_start();

        let mut last_frame = std::time::Instant::now();

        event_loop.run_app(move |event, window_target| {
            if let Some(dotrix_event) = map::event(&event) {
                handler.on_input(dotrix_event);
            }
            match event {
                // window control
                winit::event::Event::AboutToWait => {
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
                    window_target.exit();
                }
                // draw request
                winit::event::Event::RedrawRequested => {
                    handler.on_draw();
                }
                _ => {}
            }
        });
    }
     */
}

/// Window Service
pub struct Window {
    instance: Instance,
}

impl Window {
    pub fn new(instance: Instance) -> Self {
        Self { instance }
    }

    pub fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.instance.window_handle()
    }

    pub fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.instance.display_handle()
    }

    /// Set window title
    pub fn set_title(&self, title: &str) {
        self.instance.winit_window.set_title(title);
    }

    /// Returns window's resolution
    pub fn resolution(&self) -> Extent2D {
        self.instance.resolution()
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
