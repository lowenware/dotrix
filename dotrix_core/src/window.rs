//! Window service - a wrapper for [`winit::window::Window`] instance.

mod fullscreen;
mod monitor;
mod video_mode;

use crate::assets::Texture;

pub use fullscreen::Fullscreen;
use dotrix_math::{ clamp_min, Vec2, Vec2i, Vec2u };
use winit::{
    dpi::{ PhysicalPosition, PhysicalSize, Position },
    monitor:: { MonitorHandle as WinitMonitor, VideoMode as WinitVideoMode },
    window::{ self, Fullscreen as WinitFullscreen }
};
pub use monitor::Monitor;
pub use video_mode::VideoMode;
pub use window::CursorIcon as CursorIcon;
pub use window::UserAttentionType as UserAttentionType;

const NOT_SUPPORTED_ERROR: &str = "Sorry, the feature is not supported on this device.";

/// Window service - a wrapper for [`winit::window::Window`] instance.
pub struct Window {
    always_on_top: bool,
    pub (crate) close_request: bool,
    cursor_grab: bool,
    cursor_icon: CursorIcon,
    cursor_visible: bool,
    decorations: bool,
    min_inner_size: Vec2u,
    monitors: Vec<Monitor>,
    resizable: bool,
    title: String,
    /// `WINIT` window instance
    window: winit::window::Window,
}

impl Window {
    /// Service constructor from [`winit::window::Window`].
    pub (crate) fn new(window: winit::window::Window) -> Self  {
        let monitors = init_monitors(&window);
        Self {
            always_on_top: false,
            close_request: false,
            cursor_grab: false,
            cursor_icon: CursorIcon::Default,
            cursor_visible: true,
            decorations: true,
            resizable: true,
            min_inner_size: Vec2u::new(0, 0),
            monitors,
            title: String::new(),
            window,
        }
    }

    /// Check if the window will always be on top of other windows.
    pub fn always_on_top(&self) -> bool {
        self.always_on_top
    }

    /// Closes the window. It will exit the game.
    pub fn close(&mut self) {
        self.close_request = true;
    }

    /// Returns the monitor on which the window currently resides.
    pub fn current_monitor(&self) -> &Monitor {
        let mut found: Option<&Monitor> = None;
        if let Some(w_monitor) = self.window.current_monitor() {
            found = self.monitors.iter().find(|monitor| {
                monitor.name == w_monitor.name().unwrap()
            });
        }
        found.unwrap_or(&self.monitors[0])
    }

    /// Check if the cursor is grabbed (prevented from leaving the window).
    pub fn cursor_grab(&self) -> bool {
        self.cursor_grab
    }

    /// Get the cursor icon.
    pub fn cursor_icon(&self) -> CursorIcon {
        self.cursor_icon
    }

    /// Check the cursor's visibility.
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Check if the window have decorations.
    pub fn decorations(&self) -> bool {
        self.decorations
    }

    /// Check if the window is in fullscreen mode.
    pub fn fullscreen(&self) -> bool {
        self.window.fullscreen().is_some()
    }

    /// Find winit monitor based on monitor number.
    fn get_winit_monitor(&self, monitor_number: usize) -> Option<WinitMonitor> {
        if let Some(monitor) = self.monitors.get(monitor_number) {
            return self.window.available_monitors().find(|w_monitor| {
                w_monitor.name().unwrap() == monitor.name
            });
        } else {
            return None;
        }
    }

    /// Find winit video mode based on our descriptor.
    fn get_winit_video_mode(&self, vmode: VideoMode) -> Option<WinitVideoMode> {
        if let Some(monitor) = self.get_winit_monitor(vmode.monitor_number) {
            return monitor.video_modes().find(|w_vmode| {
                w_vmode.size().width == vmode.resolution.x &&
                w_vmode.size().height == vmode.resolution.y &&
                w_vmode.refresh_rate() == vmode.refresh_rate &&
                w_vmode.bit_depth() == vmode.color_depth
            });
        } else {
            return None;
        }
    }

    /// Returns the position of the top-left hand corner of the window's client
    /// area relative to the top-left hand corner of the desktop.
    pub fn inner_position(&self) -> Vec2i {
        let position = self.window.inner_position().expect(NOT_SUPPORTED_ERROR);
        Vec2i { x: position.x, y: position.y }
    }

    /// Returns the size of the window's client area in pixels.
    ///
    /// The client area is the content of the window, excluding the title bar and
    /// borders.
    pub fn inner_size(&self) -> Vec2u {
        let size = self.window.inner_size();
        Vec2u { x: size.width, y: size.height }
    }

    /// Check if the window is maximized.
    pub fn maximized(&self) -> bool {
        self.window.is_maximized()
    }

    /// Returns the minimum size of the window in pixels.
    pub fn min_inner_size(&self) -> Vec2u {
        self.min_inner_size
    }

    /// Returns a list of all available monitors.
    pub fn monitors(&self) -> &Vec<Monitor> {
        &self.monitors
    }

    /// Returns the position of the top-left hand corner of the window relative
    /// to the top-left hand corner of the desktop.
    ///
    /// Note that the top-left hand corner of the desktop is not necessarily the
    /// same as the screen. If the user uses a desktop with multiple monitors, the
    /// top-left hand corner of the desktop is the top-left hand corner of the monitor
    /// at the top-left of the desktop.
    ///
    /// The coordinates can be negative if the top-left hand corner of the window
    /// is outside of the visible screen region.
    pub fn outer_position(&self) -> Vec2i {
        let position = self.window.outer_position().expect(NOT_SUPPORTED_ERROR);
        Vec2i { x: position.x, y: position.y }
    }

    /// Returns the size of the entire window in pixels.
    ///
    /// These dimensions include the title bar and borders. If you don't want that,
    /// use `inner_size` instead.
    pub fn outer_size(&self) -> Vec2u {
        let size = self.window.outer_size();
        Vec2u { x: size.width, y: size.height }
    }

    /// Check if the window is resizable or not.
    pub fn resizable(&self) -> bool {
        self.resizable
    }

    /// Requests user attention to the window, this has no effect if the application
    /// is already focused. How requesting for user attention manifests is platform
    /// dependent, see `UserAttentionType` for details.
    ///
    /// Providing `None` will unset the request for user attention. Unsetting the
    /// request for user attention might not be done automatically by the WM when
    /// the window receives input.
    pub fn request_attention(&self, request_type: Option<UserAttentionType>) {
        self.window.request_user_attention(request_type);
    }

    pub (crate) fn request_redraw(&self) {
        self.window.request_redraw()
    }

    /// Returns the scale factor that can be used to map logical pixels to physical
    /// pixels, and vice versa.
    pub fn scale_factor(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    /// Returns the resolution of monitor on which the window currently resides.
    pub fn screen_size(&self) -> Vec2u {
        let monitor = self.window.current_monitor().expect(NOT_SUPPORTED_ERROR);
        let size = monitor.size();
        Vec2u { x: size.width, y: size.height }
    }

    /// Change whether or not the window will always be on top of other windows.
    pub fn set_always_on_top(&mut self, always_on_top: bool) {
        self.window.set_always_on_top(always_on_top);
        self.always_on_top = always_on_top;
    }

    /// Grabs the cursor, preventing it from leaving the window.
    pub fn set_cursor_grab(&mut self, grab: bool) {
        self.window.set_cursor_grab(grab).expect(NOT_SUPPORTED_ERROR);
        self.cursor_grab = grab;
    }

    /// Modifies the cursor icon of the window.
    pub fn set_cursor_icon(&mut self, icon: CursorIcon) {
        self.window.set_cursor_icon(icon);
        self.cursor_icon = icon;
    }

    /// Change the position of the cursor in window in pixel coordinates.
    pub fn set_cursor_position(&self, pos: Vec2) {
        self.window.set_cursor_position(
            Position::Physical(
                PhysicalPosition { x:  pos.x.round() as i32, y: pos.y.round() as i32 }
            )
        ).expect(NOT_SUPPORTED_ERROR);
    }

    /// Modifies the cursor's visibility.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible);
        self.cursor_visible = visible;
    }

    /// Sets the window to borderless fullscreen mode.
    ///
    /// You need to specify a monitor by it's number. Pass 0 for the primary monitor.
    ///
    /// Use `set_fullscreen(None)` to exit fullscreen.
    fn set_borderless_fullscreen(&self, monitor_number: usize) {
        let w_monitor = self.get_winit_monitor(monitor_number);
        self.window.set_fullscreen(Some(WinitFullscreen::Borderless(w_monitor)));
    }

    /// Turn window decorations on or off.
    pub fn set_decorations(&mut self, decorations: bool) {
        self.window.set_decorations(decorations);
        self.decorations = decorations;
    }

    /// Sets the window to exclusive full screen mode with the given video mode.
    ///
    /// Use `set_fullscreen(None)` to exit fullscreen.
    pub fn set_exclusive_fullscreen(&self, video_mode: VideoMode) {
        if let Some(w_video_mode) = self.get_winit_video_mode(video_mode) {
            self.window.set_fullscreen(Some(WinitFullscreen::Exclusive(w_video_mode)));
        }
    }

    /// Sets the window to fullscreen or back. You need to specify fullscreen mode.
    ///
    /// Pass `None` to end fullscreen.
    pub fn set_fullscreen(&self, fullscreen: Option<Fullscreen>) {
        if let Some(fullscreen_mode) = fullscreen {
            match fullscreen_mode {
                Fullscreen::Borderless(monitor_number) =>
                    self.set_borderless_fullscreen(monitor_number),
                Fullscreen::Exclusive(video_mode) =>
                    self.set_exclusive_fullscreen(video_mode),
            }
        } else {
            self.window.set_fullscreen(None);
        }
    }

    /// Sets the window icon. On Windows, this is typically the small icon in the
    /// top-left corner of the titlebar.
    pub fn set_icon(&self, icon: Option<&Texture>) {
        if let Some(texture) = icon {
            let icn = window::Icon::from_rgba(texture.data.clone(), texture.width, texture.height)
                .expect("Failed to open icon");
            self.window.set_window_icon(Some(icn));
        } else {
            self.window.set_window_icon(None);
        }
    }

    /// Modifies the inner size of the window.
    ///
    /// See `inner_size` for more information about the values. This automatically
    /// un-maximizes the window if it's maximized.
    pub fn set_inner_size(&self, size: Vec2u) {
        let width = clamp_min(size.x, self.min_inner_size.x);
        let height = clamp_min(size.y, self.min_inner_size.y);
        self.window.set_inner_size(PhysicalSize::new(width, height));
    }

    /// Sets the window to maximized or back.
    pub fn set_maximized(&self, maximized: bool) {
        self.window.set_maximized(maximized);
    }

    /// Sets the window to minimized or back.
    pub fn set_minimized(&mut self, minimized: bool) {
        self.window.set_minimized(minimized);
    }

    /// Sets a minimum size for the window. This automatically resize the window
    /// if it's smaller than minimum size.
    pub fn set_min_inner_size(&mut self, min_size: Vec2u) {
        self.min_inner_size = min_size;

        let min_size_physical = if min_size.x > 0 || min_size.y > 0 {
            Some(PhysicalSize::new(min_size.x, min_size.y))
        } else {
            None
        };

        self.window.set_min_inner_size(min_size_physical);

        // Resize window if the actual inner size is smaller than minimal.
        let size = self.inner_size();
        if size.x < min_size.x || size.y < min_size.y {
            self.set_inner_size(Vec2u {
                x: clamp_min(size.x, min_size.x),
                y: clamp_min(size.y, min_size.y),
            });
        }
    }

    /// Modifies the position of the window.
    ///
    /// See `outer_position` for more information about the coordinates. This automatically
    /// un-maximizes the window if it's maximized.
    pub fn set_outer_position(&self, position: Vec2i) {
        self.window.set_outer_position(PhysicalPosition::new(position.x, position.y));
    }

    /// Sets whether the window is resizable or not.
    pub fn set_resizable(&mut self, resizable: bool) {
        self.window.set_resizable(resizable);
        self.resizable = resizable;
    }

    /// Modifies the title of the window.
    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
        self.title = String::from(title);
    }

    /// Get the title of the window.
    pub fn title(&self) -> &str {
        self.title.as_str()
    }
}

fn init_monitors(window: &winit::window::Window) -> Vec<Monitor> {
    // TODO Consider:Should we include a higher monitor resolution than what is selected in the OS?
    window.available_monitors().enumerate().map(|(i, w_monitor)| {
        Monitor {
            name: w_monitor.name().unwrap(),
            number: i,
            scale_factor: w_monitor.scale_factor() as f32,
            size: Vec2u::new(w_monitor.size().width, w_monitor.size().height),
            video_modes: w_monitor.video_modes().filter_map(|vmode| {
                if vmode.size().width > w_monitor.size().width
                || vmode.size().height > w_monitor.size().height {
                    return None;
                } else {
                    return Some(
                        VideoMode {
                            color_depth: vmode.bit_depth(),
                            monitor_number: i,
                            refresh_rate: vmode.refresh_rate(),
                            resolution: Vec2u::new(vmode.size().width, vmode.size().height),
                        }
                    );
                }
            }).collect(),
        }
    }).collect()
}
