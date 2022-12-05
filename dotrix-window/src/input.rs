use dotrix_input as input;
use winit::event;

pub fn map(event: &event::Event<()>) -> Option<input::Event> {
    let input_event = match event {
        event::Event::WindowEvent {
            event: window_event,
            ..
        } => match window_event {
            event::WindowEvent::KeyboardInput { input, .. } => map_keyboard_input(input),
            event::WindowEvent::MouseInput { state, button, .. } => map_mouse_input(button, state),
            event::WindowEvent::CursorMoved { position, .. } => map_cursor_moved(position),
            event::WindowEvent::MouseWheel { delta, .. } => map_mouse_wheel(delta),
            event::WindowEvent::ModifiersChanged(state) => map_modifiers_changed(state),
            event::WindowEvent::ReceivedCharacter(chr) => map_received_character(*chr),
            event::WindowEvent::HoveredFile(path) => map_hovered_file(path),
            event::WindowEvent::HoveredFileCancelled => map_hovered_file_canceled(),
            event::WindowEvent::DroppedFile(path) => map_dropped_file(path),
            _ => return None,
        },
        event::Event::DeviceEvent {
            event: event::DeviceEvent::MouseMotion { delta },
            ..
        } => input::Event::MouseMove {
            horizontal: delta.0,
            vertical: delta.1,
        },
        _ => return None,
    };
    Some(input_event)
}

fn map_keyboard_input(input: &event::KeyboardInput) -> input::Event {
    let button = input::Button::Key {
        key_code: input.virtual_keycode.and_then(|key_code| {
            if (key_code as u32) < (input::KeyCode::Unknown as u32) {
                unsafe { Some(std::mem::transmute(key_code as u32)) }
            } else {
                None
            }
        }),
        scan_code: input.scancode,
    };
    match input.state {
        event::ElementState::Pressed => input::Event::ButtonPress { button },
        event::ElementState::Released => input::Event::ButtonRelease { button },
    }
}

fn map_modifiers_changed(state: &event::ModifiersState) -> input::Event {
    input::Event::ModifiersChange {
        modifiers: input::Modifiers::from_bits(state.bits() & input::Modifiers::all().bits())
            .unwrap(),
    }
}

fn map_received_character(character: char) -> input::Event {
    input::Event::CharacterInput { character }
}

fn map_cursor_moved(position: &winit::dpi::PhysicalPosition<f64>) -> input::Event {
    input::Event::CursorPosition {
        horizontal: position.x,
        vertical: position.y,
    }
}

fn map_mouse_wheel(delta: &event::MouseScrollDelta) -> input::Event {
    input::Event::MouseScroll {
        delta: match delta {
            event::MouseScrollDelta::LineDelta(x, y) => input::MouseScroll::Lines {
                horizontal: *x as f64,
                vertical: *y as f64,
            },
            event::MouseScrollDelta::PixelDelta(position) => input::MouseScroll::Lines {
                horizontal: position.x,
                vertical: position.y,
            },
        },
    }
}

fn map_mouse_input(button: &event::MouseButton, state: &event::ElementState) -> input::Event {
    let button = match button {
        event::MouseButton::Left => input::Button::MouseLeft,
        event::MouseButton::Right => input::Button::MouseRight,
        event::MouseButton::Middle => input::Button::MouseMiddle,
        event::MouseButton::Other(num) => input::Button::MouseOther(*num),
    };
    match state {
        event::ElementState::Pressed => input::Event::ButtonPress { button },
        event::ElementState::Released => input::Event::ButtonRelease { button },
    }
}

fn map_hovered_file(path: &std::path::Path) -> input::Event {
    input::Event::DropFileHover { path: path.into() }
}

fn map_hovered_file_canceled() -> input::Event {
    input::Event::DropFileCancel
}

fn map_dropped_file(path: &std::path::Path) -> input::Event {
    input::Event::DropFile { path: path.into() }
}

#[cfg(test)]
mod tests {
    use dotrix_input as input;
    use winit::event::{ModifiersState, VirtualKeyCode};

    #[test]
    fn dotrix_and_winit_modifiers_matches() {
        assert_eq!(ModifiersState::SHIFT.bits(), input::Modifiers::SHIFT.bits());
        assert_eq!(ModifiersState::CTRL.bits(), input::Modifiers::CTRL.bits());
        assert_eq!(ModifiersState::ALT.bits(), input::Modifiers::ALT.bits());
        assert_eq!(ModifiersState::LOGO.bits(), input::Modifiers::SUPER.bits());
    }

    #[test]
    fn dotrix_and_winit_keycodes_matches() {
        assert_eq!(
            std::mem::size_of::<VirtualKeyCode>(),
            std::mem::size_of::<input::KeyCode>()
        );
        assert_eq!(VirtualKeyCode::Key1 as u32, input::KeyCode::Key1 as u32);
        assert_eq!(VirtualKeyCode::Key2 as u32, input::KeyCode::Key2 as u32);
        assert_eq!(VirtualKeyCode::Key3 as u32, input::KeyCode::Key3 as u32);
        assert_eq!(VirtualKeyCode::Key4 as u32, input::KeyCode::Key4 as u32);
        assert_eq!(VirtualKeyCode::Key5 as u32, input::KeyCode::Key5 as u32);
        assert_eq!(VirtualKeyCode::Key6 as u32, input::KeyCode::Key6 as u32);
        assert_eq!(VirtualKeyCode::Key7 as u32, input::KeyCode::Key7 as u32);
        assert_eq!(VirtualKeyCode::Key8 as u32, input::KeyCode::Key8 as u32);
        assert_eq!(VirtualKeyCode::Key9 as u32, input::KeyCode::Key9 as u32);
        assert_eq!(VirtualKeyCode::Key0 as u32, input::KeyCode::Key0 as u32);

        assert_eq!(VirtualKeyCode::A as u32, input::KeyCode::A as u32);
        assert_eq!(VirtualKeyCode::B as u32, input::KeyCode::B as u32);
        assert_eq!(VirtualKeyCode::C as u32, input::KeyCode::C as u32);
        assert_eq!(VirtualKeyCode::D as u32, input::KeyCode::D as u32);
        assert_eq!(VirtualKeyCode::E as u32, input::KeyCode::E as u32);
        assert_eq!(VirtualKeyCode::F as u32, input::KeyCode::F as u32);
        assert_eq!(VirtualKeyCode::G as u32, input::KeyCode::G as u32);
        assert_eq!(VirtualKeyCode::H as u32, input::KeyCode::H as u32);
        assert_eq!(VirtualKeyCode::I as u32, input::KeyCode::I as u32);
        assert_eq!(VirtualKeyCode::J as u32, input::KeyCode::J as u32);
        assert_eq!(VirtualKeyCode::K as u32, input::KeyCode::K as u32);
        assert_eq!(VirtualKeyCode::L as u32, input::KeyCode::L as u32);
        assert_eq!(VirtualKeyCode::M as u32, input::KeyCode::M as u32);
        assert_eq!(VirtualKeyCode::N as u32, input::KeyCode::N as u32);
        assert_eq!(VirtualKeyCode::O as u32, input::KeyCode::O as u32);
        assert_eq!(VirtualKeyCode::P as u32, input::KeyCode::P as u32);
        assert_eq!(VirtualKeyCode::Q as u32, input::KeyCode::Q as u32);
        assert_eq!(VirtualKeyCode::R as u32, input::KeyCode::R as u32);
        assert_eq!(VirtualKeyCode::S as u32, input::KeyCode::S as u32);
        assert_eq!(VirtualKeyCode::T as u32, input::KeyCode::T as u32);
        assert_eq!(VirtualKeyCode::U as u32, input::KeyCode::U as u32);
        assert_eq!(VirtualKeyCode::V as u32, input::KeyCode::V as u32);
        assert_eq!(VirtualKeyCode::W as u32, input::KeyCode::W as u32);
        assert_eq!(VirtualKeyCode::X as u32, input::KeyCode::X as u32);
        assert_eq!(VirtualKeyCode::Y as u32, input::KeyCode::Y as u32);
        assert_eq!(VirtualKeyCode::Z as u32, input::KeyCode::Z as u32);

        assert_eq!(VirtualKeyCode::Escape as u32, input::KeyCode::Escape as u32);

        assert_eq!(VirtualKeyCode::F1 as u32, input::KeyCode::F1 as u32);
        assert_eq!(VirtualKeyCode::F2 as u32, input::KeyCode::F2 as u32);
        assert_eq!(VirtualKeyCode::F3 as u32, input::KeyCode::F3 as u32);
        assert_eq!(VirtualKeyCode::F4 as u32, input::KeyCode::F4 as u32);
        assert_eq!(VirtualKeyCode::F5 as u32, input::KeyCode::F5 as u32);
        assert_eq!(VirtualKeyCode::F6 as u32, input::KeyCode::F6 as u32);
        assert_eq!(VirtualKeyCode::F7 as u32, input::KeyCode::F7 as u32);
        assert_eq!(VirtualKeyCode::F8 as u32, input::KeyCode::F8 as u32);
        assert_eq!(VirtualKeyCode::F9 as u32, input::KeyCode::F9 as u32);
        assert_eq!(VirtualKeyCode::F10 as u32, input::KeyCode::F10 as u32);
        assert_eq!(VirtualKeyCode::F11 as u32, input::KeyCode::F11 as u32);
        assert_eq!(VirtualKeyCode::F12 as u32, input::KeyCode::F12 as u32);
        assert_eq!(VirtualKeyCode::F13 as u32, input::KeyCode::F13 as u32);
        assert_eq!(VirtualKeyCode::F14 as u32, input::KeyCode::F14 as u32);
        assert_eq!(VirtualKeyCode::F15 as u32, input::KeyCode::F15 as u32);
        assert_eq!(VirtualKeyCode::F16 as u32, input::KeyCode::F16 as u32);
        assert_eq!(VirtualKeyCode::F17 as u32, input::KeyCode::F17 as u32);
        assert_eq!(VirtualKeyCode::F18 as u32, input::KeyCode::F18 as u32);
        assert_eq!(VirtualKeyCode::F19 as u32, input::KeyCode::F19 as u32);
        assert_eq!(VirtualKeyCode::F20 as u32, input::KeyCode::F20 as u32);
        assert_eq!(VirtualKeyCode::F21 as u32, input::KeyCode::F21 as u32);
        assert_eq!(VirtualKeyCode::F22 as u32, input::KeyCode::F22 as u32);
        assert_eq!(VirtualKeyCode::F23 as u32, input::KeyCode::F23 as u32);
        assert_eq!(VirtualKeyCode::F24 as u32, input::KeyCode::F24 as u32);

        assert_eq!(
            VirtualKeyCode::Snapshot as u32,
            input::KeyCode::Snapshot as u32
        );
        assert_eq!(VirtualKeyCode::Scroll as u32, input::KeyCode::Scroll as u32);
        assert_eq!(VirtualKeyCode::Pause as u32, input::KeyCode::Pause as u32);

        assert_eq!(VirtualKeyCode::Insert as u32, input::KeyCode::Insert as u32);
        assert_eq!(VirtualKeyCode::Home as u32, input::KeyCode::Home as u32);
        assert_eq!(VirtualKeyCode::Delete as u32, input::KeyCode::Delete as u32);
        assert_eq!(VirtualKeyCode::End as u32, input::KeyCode::End as u32);
        assert_eq!(
            VirtualKeyCode::PageDown as u32,
            input::KeyCode::PageDown as u32
        );
        assert_eq!(VirtualKeyCode::PageUp as u32, input::KeyCode::PageUp as u32);

        assert_eq!(VirtualKeyCode::Left as u32, input::KeyCode::Left as u32);
        assert_eq!(VirtualKeyCode::Up as u32, input::KeyCode::Up as u32);
        assert_eq!(VirtualKeyCode::Right as u32, input::KeyCode::Right as u32);
        assert_eq!(VirtualKeyCode::Down as u32, input::KeyCode::Down as u32);

        assert_eq!(
            VirtualKeyCode::Back as u32,
            input::KeyCode::Backspace as u32
        );
        assert_eq!(VirtualKeyCode::Return as u32, input::KeyCode::Return as u32);
        assert_eq!(VirtualKeyCode::Space as u32, input::KeyCode::Space as u32);
        assert_eq!(
            VirtualKeyCode::Compose as u32,
            input::KeyCode::Compose as u32
        );
        assert_eq!(VirtualKeyCode::Caret as u32, input::KeyCode::Caret as u32);

        assert_eq!(
            VirtualKeyCode::Numlock as u32,
            input::KeyCode::Numlock as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad0 as u32,
            input::KeyCode::Numpad0 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad1 as u32,
            input::KeyCode::Numpad1 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad2 as u32,
            input::KeyCode::Numpad2 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad3 as u32,
            input::KeyCode::Numpad3 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad4 as u32,
            input::KeyCode::Numpad4 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad5 as u32,
            input::KeyCode::Numpad5 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad6 as u32,
            input::KeyCode::Numpad6 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad7 as u32,
            input::KeyCode::Numpad7 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad8 as u32,
            input::KeyCode::Numpad8 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad9 as u32,
            input::KeyCode::Numpad9 as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadAdd as u32,
            input::KeyCode::NumpadAdd as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadDivide as u32,
            input::KeyCode::NumpadDivide as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadDecimal as u32,
            input::KeyCode::NumpadDecimal as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadComma as u32,
            input::KeyCode::NumpadComma as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadEnter as u32,
            input::KeyCode::NumpadEnter as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadEquals as u32,
            input::KeyCode::NumpadEquals as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadMultiply as u32,
            input::KeyCode::NumpadMultiply as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadSubtract as u32,
            input::KeyCode::NumpadSubtract as u32
        );

        assert_eq!(VirtualKeyCode::AbntC1 as u32, input::KeyCode::AbntC1 as u32);
        assert_eq!(VirtualKeyCode::AbntC2 as u32, input::KeyCode::AbntC2 as u32);
        assert_eq!(
            VirtualKeyCode::Apostrophe as u32,
            input::KeyCode::Apostrophe as u32
        );
        assert_eq!(VirtualKeyCode::Apps as u32, input::KeyCode::Apps as u32);
        assert_eq!(
            VirtualKeyCode::Asterisk as u32,
            input::KeyCode::Asterisk as u32
        );
        assert_eq!(VirtualKeyCode::At as u32, input::KeyCode::At as u32);
        assert_eq!(VirtualKeyCode::Ax as u32, input::KeyCode::Ax as u32);
        assert_eq!(
            VirtualKeyCode::Backslash as u32,
            input::KeyCode::Backslash as u32
        );
        assert_eq!(
            VirtualKeyCode::Calculator as u32,
            input::KeyCode::Calculator as u32
        );
        assert_eq!(
            VirtualKeyCode::Capital as u32,
            input::KeyCode::Capital as u32
        );
        assert_eq!(VirtualKeyCode::Colon as u32, input::KeyCode::Colon as u32);
        assert_eq!(VirtualKeyCode::Comma as u32, input::KeyCode::Comma as u32);
        assert_eq!(
            VirtualKeyCode::Convert as u32,
            input::KeyCode::Convert as u32
        );
        assert_eq!(VirtualKeyCode::Equals as u32, input::KeyCode::Equals as u32);
        assert_eq!(VirtualKeyCode::Grave as u32, input::KeyCode::Grave as u32);
        assert_eq!(VirtualKeyCode::Kana as u32, input::KeyCode::Kana as u32);
        assert_eq!(VirtualKeyCode::Kanji as u32, input::KeyCode::Kanji as u32);
        assert_eq!(VirtualKeyCode::LAlt as u32, input::KeyCode::LAlt as u32);
        assert_eq!(
            VirtualKeyCode::LBracket as u32,
            input::KeyCode::LBracket as u32
        );
        assert_eq!(
            VirtualKeyCode::LControl as u32,
            input::KeyCode::LControl as u32
        );
        assert_eq!(VirtualKeyCode::LShift as u32, input::KeyCode::LShift as u32);
        assert_eq!(VirtualKeyCode::LWin as u32, input::KeyCode::LWin as u32);
        assert_eq!(VirtualKeyCode::Mail as u32, input::KeyCode::Mail as u32);
        assert_eq!(
            VirtualKeyCode::MediaSelect as u32,
            input::KeyCode::MediaSelect as u32
        );
        assert_eq!(
            VirtualKeyCode::MediaStop as u32,
            input::KeyCode::MediaStop as u32
        );
        assert_eq!(VirtualKeyCode::Minus as u32, input::KeyCode::Minus as u32);
        assert_eq!(VirtualKeyCode::Mute as u32, input::KeyCode::Mute as u32);
        assert_eq!(
            VirtualKeyCode::MyComputer as u32,
            input::KeyCode::MyComputer as u32
        );
        assert_eq!(
            VirtualKeyCode::NavigateForward as u32,
            input::KeyCode::NavigateForward as u32
        );
        assert_eq!(
            VirtualKeyCode::NavigateBackward as u32,
            input::KeyCode::NavigateBackward as u32
        );
        assert_eq!(
            VirtualKeyCode::NextTrack as u32,
            input::KeyCode::NextTrack as u32
        );
        assert_eq!(
            VirtualKeyCode::NoConvert as u32,
            input::KeyCode::NoConvert as u32
        );
        assert_eq!(VirtualKeyCode::OEM102 as u32, input::KeyCode::OEM102 as u32);
        assert_eq!(VirtualKeyCode::Period as u32, input::KeyCode::Period as u32);
        assert_eq!(
            VirtualKeyCode::PlayPause as u32,
            input::KeyCode::PlayPause as u32
        );
        assert_eq!(VirtualKeyCode::Plus as u32, input::KeyCode::Plus as u32);
        assert_eq!(VirtualKeyCode::Power as u32, input::KeyCode::Power as u32);
        assert_eq!(
            VirtualKeyCode::PrevTrack as u32,
            input::KeyCode::PrevTrack as u32
        );
        assert_eq!(VirtualKeyCode::RAlt as u32, input::KeyCode::RAlt as u32);
        assert_eq!(
            VirtualKeyCode::RBracket as u32,
            input::KeyCode::RBracket as u32
        );
        assert_eq!(
            VirtualKeyCode::RControl as u32,
            input::KeyCode::RControl as u32
        );
        assert_eq!(VirtualKeyCode::RShift as u32, input::KeyCode::RShift as u32);
        assert_eq!(VirtualKeyCode::RWin as u32, input::KeyCode::RWin as u32);
        assert_eq!(
            VirtualKeyCode::Semicolon as u32,
            input::KeyCode::Semicolon as u32
        );
        assert_eq!(VirtualKeyCode::Slash as u32, input::KeyCode::Slash as u32);
        assert_eq!(VirtualKeyCode::Sleep as u32, input::KeyCode::Sleep as u32);
        assert_eq!(VirtualKeyCode::Stop as u32, input::KeyCode::Stop as u32);
        assert_eq!(VirtualKeyCode::Sysrq as u32, input::KeyCode::Sysrq as u32);
        assert_eq!(VirtualKeyCode::Tab as u32, input::KeyCode::Tab as u32);
        assert_eq!(
            VirtualKeyCode::Underline as u32,
            input::KeyCode::Underline as u32
        );
        assert_eq!(
            VirtualKeyCode::Unlabeled as u32,
            input::KeyCode::Unlabeled as u32
        );
        assert_eq!(
            VirtualKeyCode::VolumeDown as u32,
            input::KeyCode::VolumeDown as u32
        );
        assert_eq!(
            VirtualKeyCode::VolumeUp as u32,
            input::KeyCode::VolumeUp as u32
        );
        assert_eq!(VirtualKeyCode::Wake as u32, input::KeyCode::Wake as u32);
        assert_eq!(
            VirtualKeyCode::WebBack as u32,
            input::KeyCode::WebBack as u32
        );
        assert_eq!(
            VirtualKeyCode::WebFavorites as u32,
            input::KeyCode::WebFavorites as u32
        );
        assert_eq!(
            VirtualKeyCode::WebForward as u32,
            input::KeyCode::WebForward as u32
        );
        assert_eq!(
            VirtualKeyCode::WebHome as u32,
            input::KeyCode::WebHome as u32
        );
        assert_eq!(
            VirtualKeyCode::WebRefresh as u32,
            input::KeyCode::WebRefresh as u32
        );
        assert_eq!(
            VirtualKeyCode::WebSearch as u32,
            input::KeyCode::WebSearch as u32
        );
        assert_eq!(
            VirtualKeyCode::WebStop as u32,
            input::KeyCode::WebStop as u32
        );
        assert_eq!(VirtualKeyCode::Yen as u32, input::KeyCode::Yen as u32);
        assert_eq!(VirtualKeyCode::Copy as u32, input::KeyCode::Copy as u32);
        assert_eq!(VirtualKeyCode::Paste as u32, input::KeyCode::Paste as u32);
        assert_eq!(VirtualKeyCode::Cut as u32, input::KeyCode::Cut as u32);
    }
}
