use super::Action;
use std::{collections::HashMap};
use strum::IntoEnumIterator;
use winit::{event::{DeviceId, ElementState, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase}};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
/// Key or button
pub enum KeyTrigger{
    Key{scancode: u32},
    MouseButton{button: MouseButton},
    None,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum KeyState {
    None,
    Pressed,
    Hold,
    Released,
}

/// Manager for input
pub struct InputManager {
    /// Mapped keys
    input_map: HashMap<KeyTrigger, Action>,
    /// Current state of actions
    key_states: HashMap<Action, KeyState>,
    /// Mouse Scrolling, value should be between -1 and 1 (but should be smaller on higher, depends on OS and device)
    scroll_delta: f32,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            input_map: HashMap::new(),
            key_states: HashMap::new(),
            scroll_delta: 0.0,
        }
    }

    /// Initialization
    pub fn register_default_keymaps(&mut self) {
        // Set key maps and states to default values
        for key_action in Action::iter() {
            self.input_map.insert(KeyTrigger::None, key_action);
            self.key_states.insert(key_action, KeyState::None);
        }

        // Map some keys on keyboard
        self.input_map.insert(KeyTrigger::Key{scancode: 57}, Action::Jump); // Space
        self.input_map.insert(KeyTrigger::Key{scancode: 17}, Action::MoveForward); // W
        self.input_map.insert(KeyTrigger::Key{scancode: 31}, Action::MoveBackward); // S
        self.input_map.insert(KeyTrigger::Key{scancode: 30}, Action::MoveLeft); // A
        self.input_map.insert(KeyTrigger::Key{scancode: 32}, Action::MoveRight); // D

         // Map some keys on mouse
        self.input_map.insert(KeyTrigger::MouseButton{button: MouseButton::Left}, Action::Shoot); // LMB
        self.input_map.insert(KeyTrigger::MouseButton{button: MouseButton::Other(1)}, Action::Ability1); // If someone has additional buttons on mouse
        self.input_map.insert(KeyTrigger::MouseButton{button: MouseButton::Other(2)}, Action::Ability2);
    }

    /// Return true when button is pressed
    pub fn get_button_down(&self, key: Action) -> bool {
        return self.key_states.get(&key).unwrap() == &KeyState::Pressed;
    }

    /// Return true when button is released
    pub fn get_button_up(&self, key: Action) -> bool {
        return self.key_states.get(&key).unwrap() == &KeyState::Released;
    }

    /// Return true button is pressed or hold
    pub fn get_button(&self, key: Action) -> bool {
        let state = self.key_states.get(&key).unwrap();
        return state == &KeyState::Pressed || state == &KeyState::Hold;
    }

    /// Returns mouse scroll delta. Value should be between -1 and 0 (but should be smaller on higher, depends on OS and device)
    pub fn get_scroll(&self) -> f32 {
        self.scroll_delta
    }

    /// Update must run before winit event loop
    pub fn update(&mut self) {
        for state in self.key_states.iter_mut() {
            match state.1 {
                KeyState::Released => {*state.1 = KeyState::None},
                KeyState::Pressed => {println!("Hold {:?} - updates every frame", &state.0); *state.1 = KeyState::Hold},
                _ => {}
            }
        }

        self.scroll_delta = 0.0;
    }

    /// Handle mouse event from winit
    pub fn handle_mouse_event(&mut self, _device_id: DeviceId, state: ElementState, button: MouseButton) {
        if !self.handle_key_trigger(KeyTrigger::MouseButton{button}, state){
            println!("{0:?} unmapped {1:?}", state, button);
        }
    }

    /// Handle mouse wheel event from winit
    pub fn handle_mouse_wheel_event(&mut self, _device_id: DeviceId, delta: MouseScrollDelta, _phase: TouchPhase) {
        match delta {
            MouseScrollDelta::LineDelta(x, y) => {self.scroll_delta = y; println!("scroll {:?}", y)}, // TODO: clamp between -1 and 1?
            _ => {println!("unmapped {:?}", delta)}
        }
    }

    /// Handle keyboard event from winit
    pub fn handle_keyboard_event(&mut self, _device_id: DeviceId, input: KeyboardInput, _is_synthetic: bool) {
        if !self.handle_key_trigger(KeyTrigger::Key{scancode: input.scancode}, input.state) {
            println!("{0:?} unmapped {1:?} {2:?}", input.state, input.scancode, input.virtual_keycode);
        }

    }

    pub fn handle_key_trigger(&mut self, key: KeyTrigger, state: ElementState) -> bool {
        if self.input_map.contains_key(&key) {
            let action = self.input_map[&key];

            match state {
                ElementState::Pressed => {
                    match self.key_states[&action] {
                        KeyState::None => {
                            println!("Press {:?}", &action);
                            self.key_states.insert(action, KeyState::Pressed);
                        },
                        _ => {}
                    }
                }
                ElementState::Released => {
                    println!("Release {:?}", &action);
                    self.key_states.insert(action, KeyState::Released);
                }
            }
            return true;
        }
        return false;
    }
}
