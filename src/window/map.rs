use crate::window::event;

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

fn keyboard_input(
    _device_id: winit::event::DeviceId,
    event: &winit::event::KeyEvent,
    _is_synthetic: bool,
) -> event::Event {
    let (key_code, scan_code) = match event.physical_key {
        // TODO: impement 1:1 mapper function
        winit::keyboard::PhysicalKey::Code(key_code) => {
            let scan_code = (key_code as u32);
            if scan_code < (event::KeyCode::Unknown as u32) {
                (
                    unsafe { Some(std::mem::transmute(key_code as u32)) },
                    scan_code,
                )
            } else {
                (None, scan_code)
            }
        }
        _ => (None, event::KeyCode::Unknown as u32),
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
                horizontal: *x as f64,
                vertical: *y as f64,
            },
            winit::event::MouseScrollDelta::PixelDelta(position) => event::MouseScroll::Lines {
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

fn map_key_event(event: winit::event::KeyEvent) -> event::KeyCode {
    match event.physical_key {
        winit::keyboard::PhysicalKey::Code(keycode) => match keycode {
            winit::keyboard::KeyCode::Digit1 => event::KeyCode::Key1,
            winit::keyboard::KeyCode::Digit2 => event::KeyCode::Key2,
            winit::keyboard::KeyCode::Digit3 => event::KeyCode::Key3,
            winit::keyboard::KeyCode::Digit4 => event::KeyCode::Key4,
            winit::keyboard::KeyCode::Digit5 => event::KeyCode::Key5,
            winit::keyboard::KeyCode::Digit6 => event::KeyCode::Key6,
            winit::keyboard::KeyCode::Digit7 => event::KeyCode::Key7,
            winit::keyboard::KeyCode::Digit8 => event::KeyCode::Key8,
            winit::keyboard::KeyCode::Digit9 => event::KeyCode::Key9,
            winit::keyboard::KeyCode::Digit0 => event::KeyCode::Key0,

            winit::keyboard::KeyCode::KeyA => event::KeyCode::A,
            winit::keyboard::KeyCode::KeyB => event::KeyCode::B,
            winit::keyboard::KeyCode::KeyC => event::KeyCode::C,
            winit::keyboard::KeyCode::KeyD => event::KeyCode::D,
            winit::keyboard::KeyCode::KeyE => event::KeyCode::E,
            winit::keyboard::KeyCode::KeyF => event::KeyCode::F,
            winit::keyboard::KeyCode::KeyG => event::KeyCode::G,
            winit::keyboard::KeyCode::KeyH => event::KeyCode::H,
            winit::keyboard::KeyCode::KeyI => event::KeyCode::I,
            winit::keyboard::KeyCode::KeyJ => event::KeyCode::J,
            winit::keyboard::KeyCode::KeyK => event::KeyCode::K,
            winit::keyboard::KeyCode::KeyL => event::KeyCode::L,
            winit::keyboard::KeyCode::KeyM => event::KeyCode::M,
            winit::keyboard::KeyCode::KeyN => event::KeyCode::N,
            winit::keyboard::KeyCode::KeyO => event::KeyCode::O,
            winit::keyboard::KeyCode::KeyP => event::KeyCode::P,
            winit::keyboard::KeyCode::KeyQ => event::KeyCode::Q,
            winit::keyboard::KeyCode::KeyR => event::KeyCode::R,
            winit::keyboard::KeyCode::KeyS => event::KeyCode::S,
            winit::keyboard::KeyCode::KeyT => event::KeyCode::T,
            winit::keyboard::KeyCode::KeyU => event::KeyCode::U,
            winit::keyboard::KeyCode::KeyV => event::KeyCode::V,
            winit::keyboard::KeyCode::KeyW => event::KeyCode::W,
            winit::keyboard::KeyCode::KeyX => event::KeyCode::X,
            winit::keyboard::KeyCode::KeyY => event::KeyCode::Y,
            winit::keyboard::KeyCode::KeyZ => event::KeyCode::Z,

            winit::keyboard::KeyCode::Escape => event::KeyCode::Escape,

            winit::keyboard::KeyCode::F1 => event::KeyCode::F1,
            winit::keyboard::KeyCode::F2 => event::KeyCode::F2,
            winit::keyboard::KeyCode::F3 => event::KeyCode::F3,
            winit::keyboard::KeyCode::F4 => event::KeyCode::F4,
            winit::keyboard::KeyCode::F5 => event::KeyCode::F5,
            winit::keyboard::KeyCode::F6 => event::KeyCode::F6,
            winit::keyboard::KeyCode::F7 => event::KeyCode::F7,
            winit::keyboard::KeyCode::F8 => event::KeyCode::F8,
            winit::keyboard::KeyCode::F9 => event::KeyCode::F9,
            winit::keyboard::KeyCode::F10 => event::KeyCode::F10,
            winit::keyboard::KeyCode::F11 => event::KeyCode::F11,
            winit::keyboard::KeyCode::F12 => event::KeyCode::F12,
            winit::keyboard::KeyCode::F13 => event::KeyCode::F13,
            winit::keyboard::KeyCode::F14 => event::KeyCode::F14,
            winit::keyboard::KeyCode::F15 => event::KeyCode::F15,
            winit::keyboard::KeyCode::F16 => event::KeyCode::F16,
            winit::keyboard::KeyCode::F17 => event::KeyCode::F17,
            winit::keyboard::KeyCode::F18 => event::KeyCode::F18,
            winit::keyboard::KeyCode::F19 => event::KeyCode::F19,
            winit::keyboard::KeyCode::F20 => event::KeyCode::F20,
            winit::keyboard::KeyCode::F21 => event::KeyCode::F21,
            winit::keyboard::KeyCode::F22 => event::KeyCode::F22,
            winit::keyboard::KeyCode::F23 => event::KeyCode::F23,
            winit::keyboard::KeyCode::F24 => event::KeyCode::F24,

            winit::keyboard::KeyCode::PrintScreen => event::KeyCode::PrintScreen,
            winit::keyboard::KeyCode::ScrollLock => event::KeyCode::ScrollLock,
            winit::keyboard::KeyCode::Pause => event::KeyCode::Pause,

            winit::keyboard::KeyCode::Insert => event::KeyCode::Insert,
            winit::keyboard::KeyCode::Home => event::KeyCode::Home,
            winit::keyboard::KeyCode::Delete => event::KeyCode::Delete,
            winit::keyboard::KeyCode::End => event::KeyCode::End,

            winit::keyboard::KeyCode::PageDown => event::KeyCode::PageDown,
            winit::keyboard::KeyCode::PageUp => event::KeyCode::PageUp,

            winit::keyboard::KeyCode::ArrowLeft => event::KeyCode::Left,
            winit::keyboard::KeyCode::ArrowUp => event::KeyCode::Up,
            winit::keyboard::KeyCode::ArrowRight => event::KeyCode::Right,
            winit::keyboard::KeyCode::ArrowDown => event::KeyCode::Down,

            winit::keyboard::KeyCode::Backspace => event::KeyCode::Backspace,
            winit::keyboard::KeyCode::Enter => event::KeyCode::Return,
            winit::keyboard::KeyCode::Space => event::KeyCode::Space,

            // winit::keyboard::KeyCode::Compose => event::KeyCode::Compose,
            // winit::keyboard::KeyCode::Caret => event::KeyCode::Caret,
            winit::keyboard::KeyCode::NumLock => event::KeyCode::Numlock,

            winit::keyboard::KeyCode::Numpad0 => event::KeyCode::Numpad0,

            winit::keyboard::KeyCode::Numpad1 => event::KeyCode::Numpad1,

            winit::keyboard::KeyCode::Numpad2 => event::KeyCode::Numpad2,

            winit::keyboard::KeyCode::Numpad3 => event::KeyCode::Numpad3,

            winit::keyboard::KeyCode::Numpad4 => event::KeyCode::Numpad4,

            winit::keyboard::KeyCode::Numpad5 => event::KeyCode::Numpad5,

            winit::keyboard::KeyCode::Numpad6 => event::KeyCode::Numpad6,

            winit::keyboard::KeyCode::Numpad7 => event::KeyCode::Numpad7,

            winit::keyboard::KeyCode::Numpad8 => event::KeyCode::Numpad8,

            winit::keyboard::KeyCode::Numpad9 => event::KeyCode::Numpad9,

            winit::keyboard::KeyCode::NumpadAdd => event::KeyCode::NumpadAdd,

            winit::keyboard::KeyCode::NumpadDivide => event::KeyCode::NumpadDivide,

            winit::keyboard::KeyCode::NumpadDecimal => event::KeyCode::NumpadDecimal,

            winit::keyboard::KeyCode::NumpadComma => event::KeyCode::NumpadComma,

            winit::keyboard::KeyCode::NumpadEnter => event::KeyCode::NumpadEnter,

            winit::keyboard::KeyCode::NumpadEqual => event::KeyCode::NumpadEquals,

            winit::keyboard::KeyCode::NumpadMultiply => event::KeyCode::NumpadMultiply,

            winit::keyboard::KeyCode::NumpadSubtract => event::KeyCode::NumpadSubtract,

            // winit::keyboard::KeyCode::AbntC1 => event::KeyCode::AbntC1,
            // winit::keyboard::KeyCode::AbntC2 => event::KeyCode::AbntC2,

            // winit::keyboard::KeyCode::Apostrophe => event::KeyCode::Apostrophe,
            // winit::keyboard::KeyCode::Apps => event::KeyCode::Apps,

            // winit::keyboard::KeyCode::Asterisk => event::KeyCode::Asterisk,
            // winit::keyboard::KeyCode::At => event::KeyCode::At,
            // winit::keyboard::KeyCode::Ax => event::KeyCode::Ax,
            winit::keyboard::KeyCode::Backslash => event::KeyCode::Backslash,

            winit::keyboard::KeyCode::LaunchApp2 => event::KeyCode::Calculator,

            // winit::keyboard::KeyCode::Capital => event::KeyCode::Capital,
            // winit::keyboard::KeyCode::Colon => event::KeyCode::Colon,
            winit::keyboard::KeyCode::Comma => event::KeyCode::Comma,

            winit::keyboard::KeyCode::Convert => event::KeyCode::Convert,
            //winit::keyboard::KeyCode::Equals => event::KeyCode::Equals,
            //winit::keyboard::KeyCode::Grave => event::KeyCode::Grave,
            //winit::keyboard::KeyCode::Kana => event::KeyCode::Kana,
            //winit::keyboard::KeyCode::Kanji => event::KeyCode::Kanji,
            winit::keyboard::KeyCode::AltLeft => event::KeyCode::LAlt,

            winit::keyboard::KeyCode::BracketLeft => event::KeyCode::LBracket,

            winit::keyboard::KeyCode::ControlLeft => event::KeyCode::LControl,
            winit::keyboard::KeyCode::ShiftLeft => event::KeyCode::LShift,
            winit::keyboard::KeyCode::SuperLeft => event::KeyCode::LWin,
            winit::keyboard::KeyCode::LaunchMail => event::KeyCode::Mail,

            winit::keyboard::KeyCode::MediaSelect => event::KeyCode::MediaSelect,

            winit::keyboard::KeyCode::MediaStop => event::KeyCode::MediaStop,
            winit::keyboard::KeyCode::Minus => event::KeyCode::Minus,
            winit::keyboard::KeyCode::AudioVolumeMute => event::KeyCode::Mute,

            winit::keyboard::KeyCode::LaunchApp1 => event::KeyCode::MyComputer,

            winit::keyboard::KeyCode::BrowserForward => event::KeyCode::NavigateForward,

            winit::keyboard::KeyCode::BrowserBack => event::KeyCode::NavigateBackward,

            winit::keyboard::KeyCode::MediaTrackNext => event::KeyCode::NextTrack,

            // winit::keyboard::KeyCode::NoConvert => event::KeyCode::NoConvert,
            // winit::keyboard::KeyCode::OEM102 => event::KeyCode::OEM102,
            winit::keyboard::KeyCode::Period => event::KeyCode::Period,

            winit::keyboard::KeyCode::MediaPlayPause => event::KeyCode::PlayPause,
            // winit::keyboard::KeyCode::Plus => event::KeyCode::Plus,
            winit::keyboard::KeyCode::Power => event::KeyCode::Power,

            winit::keyboard::KeyCode::MediaTrackPrevious => event::KeyCode::PrevTrack,
            winit::keyboard::KeyCode::AltRight => event::KeyCode::RAlt,

            winit::keyboard::KeyCode::BracketRight => event::KeyCode::RBracket,

            winit::keyboard::KeyCode::ControlRight => event::KeyCode::RControl,
            winit::keyboard::KeyCode::ShiftRight => event::KeyCode::RShift,
            winit::keyboard::KeyCode::SuperRight => event::KeyCode::RWin,

            winit::keyboard::KeyCode::Semicolon => event::KeyCode::Semicolon,
            winit::keyboard::KeyCode::Slash => event::KeyCode::Slash,
            winit::keyboard::KeyCode::Sleep => event::KeyCode::Sleep,
            winit::keyboard::KeyCode::MediaStop => event::KeyCode::Stop,
            //winit::keyboard::KeyCode::Sysrq => event::KeyCode::Sysrq,
            winit::keyboard::KeyCode::Tab => event::KeyCode::Tab,

            // winit::keyboard::KeyCode::Underline => event::KeyCode::Underline,

            // winit::keyboard::KeyCode::Unlabeled => event::KeyCode::Unlabeled,
            winit::keyboard::KeyCode::AudioVolumeDown => event::KeyCode::VolumeDown,

            winit::keyboard::KeyCode::AudioVolumeUp => event::KeyCode::VolumeUp,
            // winit::keyboard::KeyCode::Wake => event::KeyCode::Wake,
            winit::keyboard::KeyCode::BrowserBack => event::KeyCode::WebBack,

            winit::keyboard::KeyCode::BrowserFavorites => event::KeyCode::WebFavorites,

            winit::keyboard::KeyCode::BrowserForward => event::KeyCode::WebForward,

            winit::keyboard::KeyCode::BrowserHome => event::KeyCode::WebHome,

            winit::keyboard::KeyCode::BrowserRefresh => event::KeyCode::WebRefresh,

            winit::keyboard::KeyCode::BrowserSearch => event::KeyCode::WebSearch,

            winit::keyboard::KeyCode::BrowserStop => event::KeyCode::WebStop,
            // winit::keyboard::KeyCode::Yen => event::KeyCode::Yen,
            winit::keyboard::KeyCode::Copy => event::KeyCode::Copy,
            winit::keyboard::KeyCode::Paste => event::KeyCode::Paste,
            winit::keyboard::KeyCode::Cut => event::KeyCode::Cut,

            _ => event::KeyCode::Unknown,
        },
        _ => event::KeyCode::Unknown,
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
