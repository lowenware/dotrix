//! Input service, ray casting service and utils
use crate::ecs::Mut;
use dotrix_math::{Vec2, clamp};

use std::{
    collections::HashMap,
    path::{ Path, PathBuf },
};

use winit::event::{
    ElementState,
    KeyboardInput,
    MouseButton,
    MouseScrollDelta,
    WindowEvent,
};

pub use winit::event::{
    ModifiersState as Modifiers,
    VirtualKeyCode as KeyCode,
};

/// Input button abstraction
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, )] // TODO: add support for serialization
pub enum Button {
    /// Key by the code
    Key(KeyCode), // TODO: consider support for Key{scancode: u32}?
    /// Left mouse button
    MouseLeft,
    /// Right Mouse Button
    MouseRight,
    /// Middle mouse button
    MouseMiddle,
    /// Mouse button by the code
    MouseOther(u16),
}

/// State of a button
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum State {
    /// Button was activated (pressed)
    Activated,
    /// Button is hold
    Hold,
    /// Button was deactivated (released)
    Deactivated,
}

/// Input event abstraction
pub enum Event {
    /// Copy to clipboard event
    Copy,
    /// Cut to clipboard event
    Cut,
    /// Custom key event
    Key(KeyEvent),
    /// Text input event
    Text(String),
}

/// Key input event
#[derive(Debug)]
pub struct KeyEvent {
    /// Key code of the event trigger
    pub key_code: KeyCode,
    /// Key state
    pub pressed: bool,
    /// Active modifiers
    pub modifiers: Modifiers,
}

/// Input Service
///
/// Collects input events, tracks state changes and provides mapping to game actions
pub struct Input {
    mapper: Box<dyn std::any::Any + Send + Sync>,
    states: HashMap<Button, State>,
    mouse_scroll_delta: f32,
    mouse_position: Option<Vec2>,
    mouse_delta: Vec2,
    mouse_moved: bool,
    window_size: Vec2, // TODO: move to other struct or service
    /// events collector
    pub events: Vec<Event>,
    /// modifiers collector
    pub modifiers: Modifiers,
    /// Currently dropped files. Used for drag & drop.
    pub dropped_files: Option<Vec<PathBuf>>,
    /// Currently held files above the window.
    pub hovered_files: Vec<PathBuf>,
}

/// Sample input Actions enumeration
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum Action {
    /// Move forward
    MoveForward,
    /// Move Backward
    MoveBackward,
    /// Move Left
    MoveLeft,
    /// Move Right
    MoveRight
}

impl Input {
    /// Returns the status of the mapped action.
    pub fn action_state<T>(&self, action: T) -> Option<State>
    where
        Self: ActionMapper<T>,
        T: Copy + Eq + std::hash::Hash,
    {
        if let Some(button) = self.action_mapped(action) {
            if let Some(state) = self.states.get(button) {
                return Some(*state);
            }
        }
        None
    }

    /// Returns the status of the raw input
    pub fn button_state(&self, button: Button) -> Option<State> {
        self.states.get(&button).copied()
    }

    /// Checks if mapped action button is pressed
    pub fn is_action_activated<T>(&self, action: T) -> bool
    where
        Self: ActionMapper<T>,
        T: Copy + Eq + std::hash::Hash,
    {
        self.action_state(action)
            .map(|state| state == State::Activated)
            .unwrap_or(false)
    }

    /// Checks if mapped action button is released
    pub fn is_action_deactivated<T>(&self, action: T) -> bool
    where
        Self: ActionMapper<T>,
        T: Copy + Eq + std::hash::Hash,
    {
        self.action_state(action)
            .map(|state| state == State::Deactivated)
            .unwrap_or(false)
    }

    /// Checks if mapped button is pressed or hold
    pub fn is_action_hold<T>(&self, action: T) -> bool
    where
        Self: ActionMapper<T>,
        T: Copy + Eq + std::hash::Hash,
    {
        self.action_state(action)
            .map(|state| state == State::Hold || state == State::Activated)
            .unwrap_or(false)
    }

    /// Set custom [`ActionMapper`]
    pub fn set_mapper(&mut self, mapper: Box<dyn std::any::Any + Send + Sync>) {
        self.mapper = mapper;
    }

    /// Get input mapper reference
    pub fn mapper<T: 'static + Send + Sync>(&self) -> &T {
        self.mapper.downcast_ref::<T>().unwrap()
    }

    /// Get mutual mapper reference
    pub fn mapper_mut<T: 'static + Send + Sync>(&mut self) -> &mut T {
        self.mapper.downcast_mut::<T>().unwrap()
    }

    /// Set window size
    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_size = Vec2::new(width, height);
    }

    /// Mouse scroll delta
    ///
    /// Value can be positive (up) or negative (down)
    pub fn mouse_scroll(&self) -> f32 {
        self.mouse_scroll_delta
    }

    /// Current mouse position in pixel coordinates. The top-left of the window is at (0, 0)
    pub fn mouse_position(&self) -> Option<&Vec2> {
        self.mouse_position.as_ref()
    }

    /// Difference of the mouse position from the last frame in pixel coordinates
    ///
    /// The top-left of the window is at (0, 0).
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Returns true if mouse was moved this frame.
    ///
    /// This can be handy to do quick check before doing some performance costly operation.
    pub fn mouse_moved(&self) -> bool {
        self.mouse_moved
    }

    /// Normalized mouse position
    ///
    /// The top-left of the window is at (0, 0), bottom-right at (1, 1)
    pub fn mouse_position_normalized(&self) -> Vec2 {
        let (x, y) = self.mouse_position
            .as_ref()
            .map(|p| (
                    clamp(p.x / self.window_size.x, 0.0, 1.0),
                    clamp(p.y / self.window_size.y, 0.0, 1.0),
            ))
            .unwrap_or((0.0, 0.0));

        Vec2::new(x, y)
    }

    /// This method must be called periodically to update states from events
    pub(crate) fn reset(&mut self) {
        self.mouse_moved = false;
        self.mouse_delta = Vec2 { x: 0.0, y: 0.0 };
        self.mouse_scroll_delta = 0.0;

        self.states.retain(|_btn, state| match state {
            State::Activated => {
                *state = State::Hold;
                true
            }
            State::Deactivated => false,
            _ => true,
        });

        self.events.clear();
    }

    /// Handles input event
    pub(crate) fn on_event(&mut self, event: &winit::event::Event<'_, ()>) {
        if let winit::event::Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::KeyboardInput { input, .. } => self.on_keyboard_event(input),
                WindowEvent::MouseInput { state, button, .. } =>
                    self.on_mouse_click_event(*state, *button),
                WindowEvent::CursorMoved { position, .. } => self.on_cursor_moved_event(position),
                WindowEvent::MouseWheel { delta, .. } => self.on_mouse_wheel_event(delta),
                WindowEvent::Resized(size) =>
                    self.window_size = Vec2::new(size.width as f32, size.height as f32),
                WindowEvent::ModifiersChanged(input) => self.modifiers = *input,
                WindowEvent::ReceivedCharacter(chr) => {
                    if is_printable(*chr) && !self.modifiers.ctrl() && !self.modifiers.logo() {
                        if let Some(Event::Text(text)) = self.events.last_mut() {
                            text.push(*chr);
                        } else {
                            self.events.push(Event::Text(chr.to_string()));
                        }
                    }
                },
                WindowEvent::HoveredFile(buffer) => self.on_hovered_file_event(buffer),
                WindowEvent::HoveredFileCancelled => self.on_hovered_file_canceled_event(),
                WindowEvent::DroppedFile(buffer) => self.on_dropped_file_event(buffer),
                _ => {},
            }
        }

        if let winit::event::Event::DeviceEvent {
            event: winit::event::DeviceEvent::MouseMotion { delta }, ..
        } = event {
            self.on_mouse_motion_event(delta)
        }
    }

    fn on_cursor_moved_event(&mut self, position: &winit::dpi::PhysicalPosition<f64>) {
        self.mouse_moved = true;
        self.mouse_position = Some(Vec2 {
            x: position.x as f32,
            y: position.y as f32,
        });
    }

    fn on_keyboard_event(&mut self, input: &KeyboardInput) {
        if let Some(key_code) = input.virtual_keycode {
            self.events.push(Event::Key(KeyEvent {
                key_code,
                modifiers: self.modifiers,
                pressed: input.state == ElementState::Pressed,
            }));
            self.on_button_state(Button::Key(key_code), input.state);
        }
    }

    fn on_button_state(&mut self, btn: Button, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if *self.states.get(&btn).unwrap_or(&State::Deactivated) == State::Deactivated {
                    self.states.insert(btn, State::Activated);
                }
            },
            ElementState::Released => {
                self.states.insert(btn, State::Deactivated);
            },
        }
    }

    fn on_mouse_click_event(&mut self, state: ElementState, mouse_btn: winit::event::MouseButton) {
        let btn: Button = match mouse_btn {
            MouseButton::Left => Button::MouseLeft,
            MouseButton::Right => Button::MouseRight,
            MouseButton::Middle => Button::MouseMiddle,
            MouseButton::Other(num) => Button::MouseOther(num),
        };

        self.on_button_state(btn, state);
    }

    fn on_mouse_wheel_event(&mut self, delta: &MouseScrollDelta) {
        let change = match delta {
            MouseScrollDelta::LineDelta(_x, y) => *y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
        };
        self.mouse_scroll_delta += change;
    }

    fn on_mouse_motion_event(&mut self, delta: &(f64, f64)) {
        let (x, y) = *delta; // TODO: can descruct tuple as f32?
        self.mouse_delta.x += x as f32;
        self.mouse_delta.y += y as f32;
    }

    fn on_hovered_file_event(&mut self, path: &Path) {
        self.hovered_files.push(path.to_owned());
    }

    fn on_hovered_file_canceled_event(&mut self) {
        self.hovered_files.clear();
    }

    fn on_dropped_file_event(&mut self, path: &Path) {
        let path = path.to_owned();
        if let Some(dropped_files) = self.dropped_files.as_mut() {
            dropped_files.push(path);
        } else {
            self.dropped_files = Some(vec![path]);
        }
        self.hovered_files.clear();
    }
}

impl Default for Input {
    /// [`Input`] service constructor
    fn default() -> Self {
        let mapper = Box::new(Mapper::<Action>::new());
        Self {
            mapper,
            states: HashMap::new(),
            mouse_scroll_delta: 0.0,
            mouse_position: None,
            mouse_delta: Vec2::new(0.0, 0.0),
            mouse_moved: false,
            window_size: Vec2::new(0.0, 0.0),
            events: Vec::with_capacity(8),
            modifiers: Modifiers::empty(),
            dropped_files: None,
            hovered_files: Vec::new(),
        }
    }
}

/// Game action to input mapping
pub trait ActionMapper<T: Copy + Eq + std::hash::Hash> {
    /// Checks if action is mapped and returns an appropriate button
    fn action_mapped(&self, action: T) -> Option<&Button>;
}

/// Standard Mapper
pub struct Mapper<T> {
    map: HashMap<T, Button>,
}

impl<T> Mapper<T>
where
    T: Copy + Eq + std::hash::Hash
{
    /// Constructs new [`Mapper`]
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Add a new action to mapper. If action already exists, it will be overridden
    pub fn add_action(&mut self, action: T, button: Button) {
        self.map.insert(action, button);
    }

    /// Add multiple actions to mapper. Existing actions will be overridden
    pub fn add_actions(&mut self, actions: Vec<(T, Button)>) {
        for (action, button) in actions.iter() {
            self.map.insert(*action, *button);
        }
    }

    /// Get button that is binded to that action
    pub fn get_button(&self, action: T) -> Option<&Button> {
        self.map.get(&action)
    }

    /// Remove action from mapper
    pub fn remove_action(&mut self, action: T) {
        self.map.remove(&action);
    }

    /// Remove multiple actions from mapper
    pub fn remove_actions(&mut self, actions: Vec<T>) {
        for action in actions.iter() {
            self.map.remove(action);
        }
    }

    /// Removes all actions and set new ones
    pub fn set(&mut self, actions: Vec<(T, Button)>) {
        self.map.clear();
        self.add_actions(actions);
    }
}

impl<T: Copy + Eq + std::hash::Hash> Default for Mapper<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn is_printable(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

/// Input release system
pub fn release(mut input: Mut<Input>) {
    input.reset();
}
