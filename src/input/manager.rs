use super::{Action, Binding, InputConfig};
use serde::*;
use std::{collections::HashMap};
use strum::IntoEnumIterator;
use winit::{event::{DeviceId, ElementState, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase, VirtualKeyCode}};

/// Information about KeyboardKey or MouseButton
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum Button {
    Key(VirtualKeyCode), // TODO: add support for Key{scancode: u32}?
    Mouse(MouseButton),
    None,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum ButtonState {
    None,
    Pressed,
    Hold,
    Released,
}

/// Manager for input
pub struct InputManager {
    btn_map: HashMap<Button, Action>,
    btn_states: HashMap<Action, ButtonState>,
    /// Mouse Scrolling, value should be between -1 and 1 (but should be smaller on higher, depends on OS and device)
    scroll_delta: f32,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            btn_map: HashMap::new(),
            btn_states: HashMap::new(),
            scroll_delta: 0.0,
        }
    }

    /// Load bindings from config
    pub fn initialize(&mut self, config: &InputConfig) {
        // Unregister all
        for action in Action::iter() {
            self.btn_map.insert(Button::None, action);
            self.btn_states.insert(action, ButtonState::None);
        }

        // Register bindings from config
        for action_binding in config.bindings.iter() {
            let action = action_binding.0;
            let binding = action_binding.1;

            match binding.primary {
                Button::None => {},
                _ => {self.btn_map.insert(binding.primary, *action);},
            }

            match binding.secondary {
                Button::None => {},
                _ => {self.btn_map.insert(binding.secondary, *action);},
            }
        }
    }

    pub fn create_config(&mut self) -> InputConfig {
        let mut bindings: HashMap<Action, Binding> = HashMap::new();
        for button_action in self.btn_map.iter() {
            let button = button_action.0;
            let action = button_action.1;

            if bindings.contains_key(action) {
                bindings.insert(*action, Binding{primary: bindings[action].primary, secondary: *button});
            } else {
                bindings.insert(*action, Binding{primary: *button, secondary: Button::None});
            }
        }

        InputConfig {
            bindings
        }
    }

    /// Return true when button is pressed
    pub fn get_button_down(&self, action: Action) -> bool {
        self.btn_states.get(&action).unwrap() == &ButtonState::Pressed
    }

    /// Return true when button is released
    pub fn get_button_up(&self, action: Action) -> bool {
        self.btn_states.get(&action).unwrap() == &ButtonState::Released
    }

    /// Return true button is pressed or hold
    pub fn get_button(&self, action: Action) -> bool {
        let state = self.btn_states.get(&action).unwrap();
        state == &ButtonState::Pressed || state == &ButtonState::Hold
    }

    /// Returns mouse scroll delta. Value should be between -1 and 0 (but should be smaller on higher, depends on OS and device)
    pub fn get_scroll(&self) -> f32 {
        self.scroll_delta
    }

    /// Update must run before winit event loop
    pub fn update(&mut self) {
        for state in self.btn_states.iter_mut() {
            match state.1 {
                ButtonState::Released => {*state.1 = ButtonState::None},
                ButtonState::Pressed => {println!("Hold {:?} - updates every frame", &state.0); *state.1 = ButtonState::Hold},
                _ => {}
            }
        }

        self.scroll_delta = 0.0;
    }

    /// Handle mouse event from winit
    pub fn handle_mouse_event(&mut self, _device_id: DeviceId, state: ElementState, button: MouseButton) {
        if !self.handle_key_trigger(Button::Mouse(button), state){
            println!("{0:?} unmapped {1:?}", state, button);
        }
    }

    /// Handle mouse wheel event from winit
    pub fn handle_mouse_wheel_event(&mut self, _device_id: DeviceId, delta: MouseScrollDelta, _phase: TouchPhase) {
        match delta {
            MouseScrollDelta::LineDelta(_x, y) => {self.scroll_delta = y; println!("scroll {:?}", y)}, // TODO: clamp between -1 and 1?
            _ => {println!("unmapped {:?}", delta)}
        }
    }

    /// Handle keyboard event from winit
    pub fn handle_keyboard_event(&mut self, _device_id: DeviceId, input: KeyboardInput, _is_synthetic: bool) {
        if input.virtual_keycode.is_some()
            && self.handle_key_trigger(Button::Key(input.virtual_keycode.unwrap()), input.state) {
            return;
        }

        println!("{0:?} unmapped {1:?} {2:?}", input.state, input.scancode, input.virtual_keycode);
    }

    fn handle_key_trigger(&mut self, key: Button, state: ElementState) -> bool {
        if self.btn_map.contains_key(&key) {
            let action = self.btn_map[&key];

            match state {
                ElementState::Pressed => {
                    if let ButtonState::None = self.btn_states[&action] {
                        println!("Press {:?}", &action);
                        self.btn_states.insert(action, ButtonState::Pressed);
                    }
                }
                ElementState::Released => {
                    println!("Release {:?}", &action);
                    self.btn_states.insert(action, ButtonState::Released);
                }
            }
            return true;
        }
        false
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
