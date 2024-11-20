use crate::window::event;

/*
pub fn event(event: &winit::event::Event<()>) -> Option<event::Event> {
    let input_event = match event {
        winit::event::Event::WindowEvent {
            event: window_event,
            ..
        } => match window_event {
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => keyboard_input(*device_id, event, *is_synthetic),
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                mouse_input(button, state)
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => cursor_moved(position),
            winit::event::WindowEvent::MouseWheel { delta, .. } => mouse_wheel(delta),
            winit::event::WindowEvent::ModifiersChanged(modifiers) => modifiers_changed(modifiers),
            winit::event::WindowEvent::HoveredFile(path) => hovered_file(path),
            winit::event::WindowEvent::HoveredFileCancelled => hovered_file_canceled(),
            winit::event::WindowEvent::DroppedFile(path) => dropped_file(path),
            _ => return None,
        },
        winit::event::Event::DeviceEvent {
            event: winit::event::DeviceEvent::MouseMotion { delta },
            ..
        } => event::Event::MouseMove {
            horizontal: delta.0,
            vertical: delta.1,
        },
        _ => return None,
    };
    Some(input_event)
}
*/

pub fn keyboard_input(
    _device_id: winit::event::DeviceId,
    event: &winit::event::KeyEvent,
    _is_synthetic: bool,
) -> event::Event {
    let (key_code, scan_code) = match event.physical_key {
        // TODO: impement 1:1 mapper function
        winit::keyboard::PhysicalKey::Code(key_code) => {
            let scan_code = key_code as u32;
            if scan_code < (event::Key::Unknown as u32) {
                (
                    unsafe { Some(std::mem::transmute(key_code as u32)) },
                    scan_code,
                )
            } else {
                (None, scan_code)
            }
        }
        _ => (None, event::Key::Unknown as u32),
    };

    let button = event::Button::Key {
        key_code,
        scan_code,
    };
    match event.state {
        winit::event::ElementState::Pressed => event::Event::ButtonPress {
            button,
            text: event.text.as_ref().map(|smol_str| smol_str.to_string()),
        },
        winit::event::ElementState::Released => event::Event::ButtonRelease { button },
    }
}

fn modifiers_changed(modifiers: &winit::event::Modifiers) -> event::Event {
    event::Event::ModifiersChange {
        modifiers: event::Modifiers::from_bits(
            modifiers.state().bits() & event::Modifiers::all().bits(),
        )
        .unwrap(),
    }
}

fn cursor_moved(position: &winit::dpi::PhysicalPosition<f64>) -> event::Event {
    event::Event::CursorPosition {
        horizontal: position.x,
        vertical: position.y,
    }
}

fn mouse_wheel(delta: &winit::event::MouseScrollDelta) -> event::Event {
    event::Event::MouseScroll {
        delta: match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => event::MouseScroll::Lines {
                horizontal: *x,
                vertical: *y,
            },
            winit::event::MouseScrollDelta::PixelDelta(position) => event::MouseScroll::Pixels {
                horizontal: position.x,
                vertical: position.y,
            },
        },
    }
}

fn mouse_input(
    button: &winit::event::MouseButton,
    state: &winit::event::ElementState,
) -> event::Event {
    let button = match button {
        winit::event::MouseButton::Left => event::Button::MouseLeft,
        winit::event::MouseButton::Right => event::Button::MouseRight,
        winit::event::MouseButton::Middle => event::Button::MouseMiddle,
        winit::event::MouseButton::Forward => event::Button::Forward,
        winit::event::MouseButton::Back => event::Button::Back,
        winit::event::MouseButton::Other(num) => event::Button::MouseOther(*num),
    };
    match state {
        winit::event::ElementState::Pressed => event::Event::ButtonPress { button, text: None },
        winit::event::ElementState::Released => event::Event::ButtonRelease { button },
    }
}

fn hovered_file(path: &std::path::Path) -> event::Event {
    event::Event::DragAndDrop {
        target: event::DragAndDrop::FileDragged { path: path.into() },
    }
}

fn hovered_file_canceled() -> event::Event {
    event::Event::DragAndDrop {
        target: event::DragAndDrop::Canceled,
    }
}

fn dropped_file(path: &std::path::Path) -> event::Event {
    event::Event::DragAndDrop {
        target: event::DragAndDrop::FileDropped { path: path.into() },
    }
}

fn map_key_event(event: winit::event::KeyEvent) -> event::Key {
    match event.physical_key {
        winit::keyboard::PhysicalKey::Code(keycode) => match keycode {
            winit::keyboard::KeyCode::Digit1 => event::Key::Key1,
            winit::keyboard::KeyCode::Digit2 => event::Key::Key2,
            winit::keyboard::KeyCode::Digit3 => event::Key::Key3,
            winit::keyboard::KeyCode::Digit4 => event::Key::Key4,
            winit::keyboard::KeyCode::Digit5 => event::Key::Key5,
            winit::keyboard::KeyCode::Digit6 => event::Key::Key6,
            winit::keyboard::KeyCode::Digit7 => event::Key::Key7,
            winit::keyboard::KeyCode::Digit8 => event::Key::Key8,
            winit::keyboard::KeyCode::Digit9 => event::Key::Key9,
            winit::keyboard::KeyCode::Digit0 => event::Key::Key0,

            winit::keyboard::KeyCode::KeyA => event::Key::A,
            winit::keyboard::KeyCode::KeyB => event::Key::B,
            winit::keyboard::KeyCode::KeyC => event::Key::C,
            winit::keyboard::KeyCode::KeyD => event::Key::D,
            winit::keyboard::KeyCode::KeyE => event::Key::E,
            winit::keyboard::KeyCode::KeyF => event::Key::F,
            winit::keyboard::KeyCode::KeyG => event::Key::G,
            winit::keyboard::KeyCode::KeyH => event::Key::H,
            winit::keyboard::KeyCode::KeyI => event::Key::I,
            winit::keyboard::KeyCode::KeyJ => event::Key::J,
            winit::keyboard::KeyCode::KeyK => event::Key::K,
            winit::keyboard::KeyCode::KeyL => event::Key::L,
            winit::keyboard::KeyCode::KeyM => event::Key::M,
            winit::keyboard::KeyCode::KeyN => event::Key::N,
            winit::keyboard::KeyCode::KeyO => event::Key::O,
            winit::keyboard::KeyCode::KeyP => event::Key::P,
            winit::keyboard::KeyCode::KeyQ => event::Key::Q,
            winit::keyboard::KeyCode::KeyR => event::Key::R,
            winit::keyboard::KeyCode::KeyS => event::Key::S,
            winit::keyboard::KeyCode::KeyT => event::Key::T,
            winit::keyboard::KeyCode::KeyU => event::Key::U,
            winit::keyboard::KeyCode::KeyV => event::Key::V,
            winit::keyboard::KeyCode::KeyW => event::Key::W,
            winit::keyboard::KeyCode::KeyX => event::Key::X,
            winit::keyboard::KeyCode::KeyY => event::Key::Y,
            winit::keyboard::KeyCode::KeyZ => event::Key::Z,

            winit::keyboard::KeyCode::Escape => event::Key::Escape,

            winit::keyboard::KeyCode::F1 => event::Key::F1,
            winit::keyboard::KeyCode::F2 => event::Key::F2,
            winit::keyboard::KeyCode::F3 => event::Key::F3,
            winit::keyboard::KeyCode::F4 => event::Key::F4,
            winit::keyboard::KeyCode::F5 => event::Key::F5,
            winit::keyboard::KeyCode::F6 => event::Key::F6,
            winit::keyboard::KeyCode::F7 => event::Key::F7,
            winit::keyboard::KeyCode::F8 => event::Key::F8,
            winit::keyboard::KeyCode::F9 => event::Key::F9,
            winit::keyboard::KeyCode::F10 => event::Key::F10,
            winit::keyboard::KeyCode::F11 => event::Key::F11,
            winit::keyboard::KeyCode::F12 => event::Key::F12,
            winit::keyboard::KeyCode::F13 => event::Key::F13,
            winit::keyboard::KeyCode::F14 => event::Key::F14,
            winit::keyboard::KeyCode::F15 => event::Key::F15,
            winit::keyboard::KeyCode::F16 => event::Key::F16,
            winit::keyboard::KeyCode::F17 => event::Key::F17,
            winit::keyboard::KeyCode::F18 => event::Key::F18,
            winit::keyboard::KeyCode::F19 => event::Key::F19,
            winit::keyboard::KeyCode::F20 => event::Key::F20,
            winit::keyboard::KeyCode::F21 => event::Key::F21,
            winit::keyboard::KeyCode::F22 => event::Key::F22,
            winit::keyboard::KeyCode::F23 => event::Key::F23,
            winit::keyboard::KeyCode::F24 => event::Key::F24,

            winit::keyboard::KeyCode::PrintScreen => event::Key::PrintScreen,
            winit::keyboard::KeyCode::ScrollLock => event::Key::ScrollLock,
            winit::keyboard::KeyCode::Pause => event::Key::Pause,

            winit::keyboard::KeyCode::Insert => event::Key::Insert,
            winit::keyboard::KeyCode::Home => event::Key::Home,
            winit::keyboard::KeyCode::Delete => event::Key::Delete,
            winit::keyboard::KeyCode::End => event::Key::End,

            winit::keyboard::KeyCode::PageDown => event::Key::PageDown,
            winit::keyboard::KeyCode::PageUp => event::Key::PageUp,

            winit::keyboard::KeyCode::ArrowLeft => event::Key::Left,
            winit::keyboard::KeyCode::ArrowUp => event::Key::Up,
            winit::keyboard::KeyCode::ArrowRight => event::Key::Right,
            winit::keyboard::KeyCode::ArrowDown => event::Key::Down,

            winit::keyboard::KeyCode::Backspace => event::Key::Backspace,
            winit::keyboard::KeyCode::Enter => event::Key::Return,
            winit::keyboard::KeyCode::Space => event::Key::Space,

            // winit::keyboard::KeyCode::Compose => event::Key::Compose,
            // winit::keyboard::KeyCode::Caret => event::Key::Caret,
            winit::keyboard::KeyCode::NumLock => event::Key::Numlock,

            winit::keyboard::KeyCode::Numpad0 => event::Key::Numpad0,

            winit::keyboard::KeyCode::Numpad1 => event::Key::Numpad1,

            winit::keyboard::KeyCode::Numpad2 => event::Key::Numpad2,

            winit::keyboard::KeyCode::Numpad3 => event::Key::Numpad3,

            winit::keyboard::KeyCode::Numpad4 => event::Key::Numpad4,

            winit::keyboard::KeyCode::Numpad5 => event::Key::Numpad5,

            winit::keyboard::KeyCode::Numpad6 => event::Key::Numpad6,

            winit::keyboard::KeyCode::Numpad7 => event::Key::Numpad7,

            winit::keyboard::KeyCode::Numpad8 => event::Key::Numpad8,

            winit::keyboard::KeyCode::Numpad9 => event::Key::Numpad9,

            winit::keyboard::KeyCode::NumpadAdd => event::Key::NumpadAdd,

            winit::keyboard::KeyCode::NumpadDivide => event::Key::NumpadDivide,

            winit::keyboard::KeyCode::NumpadDecimal => event::Key::NumpadDecimal,

            winit::keyboard::KeyCode::NumpadComma => event::Key::NumpadComma,

            winit::keyboard::KeyCode::NumpadEnter => event::Key::NumpadEnter,

            winit::keyboard::KeyCode::NumpadEqual => event::Key::NumpadEquals,

            winit::keyboard::KeyCode::NumpadMultiply => event::Key::NumpadMultiply,

            winit::keyboard::KeyCode::NumpadSubtract => event::Key::NumpadSubtract,

            // winit::keyboard::KeyCode::AbntC1 => event::Key::AbntC1,
            // winit::keyboard::KeyCode::AbntC2 => event::Key::AbntC2,

            // winit::keyboard::KeyCode::Apostrophe => event::Key::Apostrophe,
            // winit::keyboard::KeyCode::Apps => event::Key::Apps,

            // winit::keyboard::KeyCode::Asterisk => event::Key::Asterisk,
            // winit::keyboard::KeyCode::At => event::Key::At,
            // winit::keyboard::KeyCode::Ax => event::Key::Ax,
            winit::keyboard::KeyCode::Backslash => event::Key::Backslash,

            winit::keyboard::KeyCode::LaunchApp2 => event::Key::Calculator,

            // winit::keyboard::KeyCode::Capital => event::Key::Capital,
            // winit::keyboard::KeyCode::Colon => event::Key::Colon,
            winit::keyboard::KeyCode::Comma => event::Key::Comma,

            winit::keyboard::KeyCode::Convert => event::Key::Convert,
            //winit::keyboard::KeyCode::Equals => event::Key::Equals,
            //winit::keyboard::KeyCode::Grave => event::Key::Grave,
            //winit::keyboard::KeyCode::Kana => event::Key::Kana,
            //winit::keyboard::KeyCode::Kanji => event::Key::Kanji,
            winit::keyboard::KeyCode::AltLeft => event::Key::LAlt,

            winit::keyboard::KeyCode::BracketLeft => event::Key::LBracket,

            winit::keyboard::KeyCode::ControlLeft => event::Key::LControl,
            winit::keyboard::KeyCode::ShiftLeft => event::Key::LShift,
            winit::keyboard::KeyCode::SuperLeft => event::Key::LWin,
            winit::keyboard::KeyCode::LaunchMail => event::Key::Mail,

            winit::keyboard::KeyCode::MediaSelect => event::Key::MediaSelect,

            winit::keyboard::KeyCode::MediaStop => event::Key::MediaStop,
            winit::keyboard::KeyCode::Minus => event::Key::Minus,
            winit::keyboard::KeyCode::AudioVolumeMute => event::Key::Mute,

            winit::keyboard::KeyCode::LaunchApp1 => event::Key::MyComputer,

            winit::keyboard::KeyCode::BrowserForward => event::Key::NavigateForward,

            winit::keyboard::KeyCode::BrowserBack => event::Key::NavigateBackward,

            winit::keyboard::KeyCode::MediaTrackNext => event::Key::NextTrack,

            // winit::keyboard::KeyCode::NoConvert => event::Key::NoConvert,
            // winit::keyboard::KeyCode::OEM102 => event::Key::OEM102,
            winit::keyboard::KeyCode::Period => event::Key::Period,

            winit::keyboard::KeyCode::MediaPlayPause => event::Key::PlayPause,
            // winit::keyboard::KeyCode::Plus => event::Key::Plus,
            winit::keyboard::KeyCode::Power => event::Key::Power,

            winit::keyboard::KeyCode::MediaTrackPrevious => event::Key::PrevTrack,
            winit::keyboard::KeyCode::AltRight => event::Key::RAlt,

            winit::keyboard::KeyCode::BracketRight => event::Key::RBracket,

            winit::keyboard::KeyCode::ControlRight => event::Key::RControl,
            winit::keyboard::KeyCode::ShiftRight => event::Key::RShift,
            winit::keyboard::KeyCode::SuperRight => event::Key::RWin,

            winit::keyboard::KeyCode::Semicolon => event::Key::Semicolon,
            winit::keyboard::KeyCode::Slash => event::Key::Slash,
            winit::keyboard::KeyCode::Sleep => event::Key::Sleep,
            winit::keyboard::KeyCode::MediaStop => event::Key::Stop,
            //winit::keyboard::KeyCode::Sysrq => event::Key::Sysrq,
            winit::keyboard::KeyCode::Tab => event::Key::Tab,

            // winit::keyboard::KeyCode::Underline => event::Key::Underline,

            // winit::keyboard::KeyCode::Unlabeled => event::Key::Unlabeled,
            winit::keyboard::KeyCode::AudioVolumeDown => event::Key::VolumeDown,

            winit::keyboard::KeyCode::AudioVolumeUp => event::Key::VolumeUp,
            // winit::keyboard::KeyCode::Wake => event::Key::Wake,
            winit::keyboard::KeyCode::BrowserBack => event::Key::WebBack,

            winit::keyboard::KeyCode::BrowserFavorites => event::Key::WebFavorites,

            winit::keyboard::KeyCode::BrowserForward => event::Key::WebForward,

            winit::keyboard::KeyCode::BrowserHome => event::Key::WebHome,

            winit::keyboard::KeyCode::BrowserRefresh => event::Key::WebRefresh,

            winit::keyboard::KeyCode::BrowserSearch => event::Key::WebSearch,

            winit::keyboard::KeyCode::BrowserStop => event::Key::WebStop,
            // winit::keyboard::KeyCode::Yen => event::Key::Yen,
            winit::keyboard::KeyCode::Copy => event::Key::Copy,
            winit::keyboard::KeyCode::Paste => event::Key::Paste,
            winit::keyboard::KeyCode::Cut => event::Key::Cut,

            _ => event::Key::Unknown,
        },
        _ => event::Key::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use crate::window::event;
    use winit::keyboard::ModifiersState;

    #[test]
    fn dotrix_and_winit_modifiers_matches() {
        assert_eq!(ModifiersState::SHIFT.bits(), event::Modifiers::SHIFT.bits());
        assert_eq!(
            ModifiersState::CONTROL.bits(),
            event::Modifiers::CTRL.bits()
        );
        assert_eq!(ModifiersState::ALT.bits(), event::Modifiers::ALT.bits());
        assert_eq!(ModifiersState::SUPER.bits(), event::Modifiers::SUPER.bits());
    }
}
