use crate::window::event;

pub fn event(event: &winit::event::Event<()>) -> Option<event::Event> {
    let input_event = match event {
        winit::event::Event::WindowEvent {
            event: window_event,
            ..
        } => match window_event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => keyboard_input(input),
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                mouse_input(button, state)
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => cursor_moved(position),
            winit::event::WindowEvent::MouseWheel { delta, .. } => mouse_wheel(delta),
            winit::event::WindowEvent::ModifiersChanged(state) => modifiers_changed(state),
            winit::event::WindowEvent::ReceivedCharacter(chr) => received_character(*chr),
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

fn keyboard_input(input: &winit::event::KeyboardInput) -> event::Event {
    let button = event::Button::Key {
        key_code: input.virtual_keycode.and_then(|key_code| {
            if (key_code as u32) < (event::KeyCode::Unknown as u32) {
                unsafe { Some(std::mem::transmute(key_code as u32)) }
            } else {
                None
            }
        }),
        scan_code: input.scancode,
    };
    match input.state {
        winit::event::ElementState::Pressed => event::Event::ButtonPress { button },
        winit::event::ElementState::Released => event::Event::ButtonRelease { button },
    }
}

fn modifiers_changed(state: &winit::event::ModifiersState) -> event::Event {
    event::Event::ModifiersChange {
        modifiers: event::Modifiers::from_bits(state.bits() & event::Modifiers::all().bits())
            .unwrap(),
    }
}

fn received_character(character: char) -> event::Event {
    event::Event::CharacterInput { character }
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
        winit::event::MouseButton::Other(num) => event::Button::MouseOther(*num),
    };
    match state {
        winit::event::ElementState::Pressed => event::Event::ButtonPress { button },
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

#[cfg(test)]
mod tests {
    use crate::window::event;
    use winit::event::{ModifiersState, VirtualKeyCode};

    #[test]
    fn dotrix_and_winit_modifiers_matches() {
        assert_eq!(ModifiersState::SHIFT.bits(), event::Modifiers::SHIFT.bits());
        assert_eq!(ModifiersState::CTRL.bits(), event::Modifiers::CTRL.bits());
        assert_eq!(ModifiersState::ALT.bits(), event::Modifiers::ALT.bits());
        assert_eq!(ModifiersState::LOGO.bits(), event::Modifiers::SUPER.bits());
    }

    #[test]
    fn dotrix_and_winit_keycodes_matches() {
        assert_eq!(
            std::mem::size_of::<VirtualKeyCode>(),
            std::mem::size_of::<event::KeyCode>()
        );
        assert_eq!(VirtualKeyCode::Key1 as u32, event::KeyCode::Key1 as u32);
        assert_eq!(VirtualKeyCode::Key2 as u32, event::KeyCode::Key2 as u32);
        assert_eq!(VirtualKeyCode::Key3 as u32, event::KeyCode::Key3 as u32);
        assert_eq!(VirtualKeyCode::Key4 as u32, event::KeyCode::Key4 as u32);
        assert_eq!(VirtualKeyCode::Key5 as u32, event::KeyCode::Key5 as u32);
        assert_eq!(VirtualKeyCode::Key6 as u32, event::KeyCode::Key6 as u32);
        assert_eq!(VirtualKeyCode::Key7 as u32, event::KeyCode::Key7 as u32);
        assert_eq!(VirtualKeyCode::Key8 as u32, event::KeyCode::Key8 as u32);
        assert_eq!(VirtualKeyCode::Key9 as u32, event::KeyCode::Key9 as u32);
        assert_eq!(VirtualKeyCode::Key0 as u32, event::KeyCode::Key0 as u32);

        assert_eq!(VirtualKeyCode::A as u32, event::KeyCode::A as u32);
        assert_eq!(VirtualKeyCode::B as u32, event::KeyCode::B as u32);
        assert_eq!(VirtualKeyCode::C as u32, event::KeyCode::C as u32);
        assert_eq!(VirtualKeyCode::D as u32, event::KeyCode::D as u32);
        assert_eq!(VirtualKeyCode::E as u32, event::KeyCode::E as u32);
        assert_eq!(VirtualKeyCode::F as u32, event::KeyCode::F as u32);
        assert_eq!(VirtualKeyCode::G as u32, event::KeyCode::G as u32);
        assert_eq!(VirtualKeyCode::H as u32, event::KeyCode::H as u32);
        assert_eq!(VirtualKeyCode::I as u32, event::KeyCode::I as u32);
        assert_eq!(VirtualKeyCode::J as u32, event::KeyCode::J as u32);
        assert_eq!(VirtualKeyCode::K as u32, event::KeyCode::K as u32);
        assert_eq!(VirtualKeyCode::L as u32, event::KeyCode::L as u32);
        assert_eq!(VirtualKeyCode::M as u32, event::KeyCode::M as u32);
        assert_eq!(VirtualKeyCode::N as u32, event::KeyCode::N as u32);
        assert_eq!(VirtualKeyCode::O as u32, event::KeyCode::O as u32);
        assert_eq!(VirtualKeyCode::P as u32, event::KeyCode::P as u32);
        assert_eq!(VirtualKeyCode::Q as u32, event::KeyCode::Q as u32);
        assert_eq!(VirtualKeyCode::R as u32, event::KeyCode::R as u32);
        assert_eq!(VirtualKeyCode::S as u32, event::KeyCode::S as u32);
        assert_eq!(VirtualKeyCode::T as u32, event::KeyCode::T as u32);
        assert_eq!(VirtualKeyCode::U as u32, event::KeyCode::U as u32);
        assert_eq!(VirtualKeyCode::V as u32, event::KeyCode::V as u32);
        assert_eq!(VirtualKeyCode::W as u32, event::KeyCode::W as u32);
        assert_eq!(VirtualKeyCode::X as u32, event::KeyCode::X as u32);
        assert_eq!(VirtualKeyCode::Y as u32, event::KeyCode::Y as u32);
        assert_eq!(VirtualKeyCode::Z as u32, event::KeyCode::Z as u32);

        assert_eq!(VirtualKeyCode::Escape as u32, event::KeyCode::Escape as u32);

        assert_eq!(VirtualKeyCode::F1 as u32, event::KeyCode::F1 as u32);
        assert_eq!(VirtualKeyCode::F2 as u32, event::KeyCode::F2 as u32);
        assert_eq!(VirtualKeyCode::F3 as u32, event::KeyCode::F3 as u32);
        assert_eq!(VirtualKeyCode::F4 as u32, event::KeyCode::F4 as u32);
        assert_eq!(VirtualKeyCode::F5 as u32, event::KeyCode::F5 as u32);
        assert_eq!(VirtualKeyCode::F6 as u32, event::KeyCode::F6 as u32);
        assert_eq!(VirtualKeyCode::F7 as u32, event::KeyCode::F7 as u32);
        assert_eq!(VirtualKeyCode::F8 as u32, event::KeyCode::F8 as u32);
        assert_eq!(VirtualKeyCode::F9 as u32, event::KeyCode::F9 as u32);
        assert_eq!(VirtualKeyCode::F10 as u32, event::KeyCode::F10 as u32);
        assert_eq!(VirtualKeyCode::F11 as u32, event::KeyCode::F11 as u32);
        assert_eq!(VirtualKeyCode::F12 as u32, event::KeyCode::F12 as u32);
        assert_eq!(VirtualKeyCode::F13 as u32, event::KeyCode::F13 as u32);
        assert_eq!(VirtualKeyCode::F14 as u32, event::KeyCode::F14 as u32);
        assert_eq!(VirtualKeyCode::F15 as u32, event::KeyCode::F15 as u32);
        assert_eq!(VirtualKeyCode::F16 as u32, event::KeyCode::F16 as u32);
        assert_eq!(VirtualKeyCode::F17 as u32, event::KeyCode::F17 as u32);
        assert_eq!(VirtualKeyCode::F18 as u32, event::KeyCode::F18 as u32);
        assert_eq!(VirtualKeyCode::F19 as u32, event::KeyCode::F19 as u32);
        assert_eq!(VirtualKeyCode::F20 as u32, event::KeyCode::F20 as u32);
        assert_eq!(VirtualKeyCode::F21 as u32, event::KeyCode::F21 as u32);
        assert_eq!(VirtualKeyCode::F22 as u32, event::KeyCode::F22 as u32);
        assert_eq!(VirtualKeyCode::F23 as u32, event::KeyCode::F23 as u32);
        assert_eq!(VirtualKeyCode::F24 as u32, event::KeyCode::F24 as u32);

        assert_eq!(
            VirtualKeyCode::Snapshot as u32,
            event::KeyCode::Snapshot as u32
        );
        assert_eq!(VirtualKeyCode::Scroll as u32, event::KeyCode::Scroll as u32);
        assert_eq!(VirtualKeyCode::Pause as u32, event::KeyCode::Pause as u32);

        assert_eq!(VirtualKeyCode::Insert as u32, event::KeyCode::Insert as u32);
        assert_eq!(VirtualKeyCode::Home as u32, event::KeyCode::Home as u32);
        assert_eq!(VirtualKeyCode::Delete as u32, event::KeyCode::Delete as u32);
        assert_eq!(VirtualKeyCode::End as u32, event::KeyCode::End as u32);
        assert_eq!(
            VirtualKeyCode::PageDown as u32,
            event::KeyCode::PageDown as u32
        );
        assert_eq!(VirtualKeyCode::PageUp as u32, event::KeyCode::PageUp as u32);

        assert_eq!(VirtualKeyCode::Left as u32, event::KeyCode::Left as u32);
        assert_eq!(VirtualKeyCode::Up as u32, event::KeyCode::Up as u32);
        assert_eq!(VirtualKeyCode::Right as u32, event::KeyCode::Right as u32);
        assert_eq!(VirtualKeyCode::Down as u32, event::KeyCode::Down as u32);

        assert_eq!(
            VirtualKeyCode::Back as u32,
            event::KeyCode::Backspace as u32
        );
        assert_eq!(VirtualKeyCode::Return as u32, event::KeyCode::Return as u32);
        assert_eq!(VirtualKeyCode::Space as u32, event::KeyCode::Space as u32);
        assert_eq!(
            VirtualKeyCode::Compose as u32,
            event::KeyCode::Compose as u32
        );
        assert_eq!(VirtualKeyCode::Caret as u32, event::KeyCode::Caret as u32);

        assert_eq!(
            VirtualKeyCode::Numlock as u32,
            event::KeyCode::Numlock as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad0 as u32,
            event::KeyCode::Numpad0 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad1 as u32,
            event::KeyCode::Numpad1 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad2 as u32,
            event::KeyCode::Numpad2 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad3 as u32,
            event::KeyCode::Numpad3 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad4 as u32,
            event::KeyCode::Numpad4 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad5 as u32,
            event::KeyCode::Numpad5 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad6 as u32,
            event::KeyCode::Numpad6 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad7 as u32,
            event::KeyCode::Numpad7 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad8 as u32,
            event::KeyCode::Numpad8 as u32
        );
        assert_eq!(
            VirtualKeyCode::Numpad9 as u32,
            event::KeyCode::Numpad9 as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadAdd as u32,
            event::KeyCode::NumpadAdd as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadDivide as u32,
            event::KeyCode::NumpadDivide as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadDecimal as u32,
            event::KeyCode::NumpadDecimal as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadComma as u32,
            event::KeyCode::NumpadComma as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadEnter as u32,
            event::KeyCode::NumpadEnter as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadEquals as u32,
            event::KeyCode::NumpadEquals as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadMultiply as u32,
            event::KeyCode::NumpadMultiply as u32
        );
        assert_eq!(
            VirtualKeyCode::NumpadSubtract as u32,
            event::KeyCode::NumpadSubtract as u32
        );

        assert_eq!(VirtualKeyCode::AbntC1 as u32, event::KeyCode::AbntC1 as u32);
        assert_eq!(VirtualKeyCode::AbntC2 as u32, event::KeyCode::AbntC2 as u32);
        assert_eq!(
            VirtualKeyCode::Apostrophe as u32,
            event::KeyCode::Apostrophe as u32
        );
        assert_eq!(VirtualKeyCode::Apps as u32, event::KeyCode::Apps as u32);
        assert_eq!(
            VirtualKeyCode::Asterisk as u32,
            event::KeyCode::Asterisk as u32
        );
        assert_eq!(VirtualKeyCode::At as u32, event::KeyCode::At as u32);
        assert_eq!(VirtualKeyCode::Ax as u32, event::KeyCode::Ax as u32);
        assert_eq!(
            VirtualKeyCode::Backslash as u32,
            event::KeyCode::Backslash as u32
        );
        assert_eq!(
            VirtualKeyCode::Calculator as u32,
            event::KeyCode::Calculator as u32
        );
        assert_eq!(
            VirtualKeyCode::Capital as u32,
            event::KeyCode::Capital as u32
        );
        assert_eq!(VirtualKeyCode::Colon as u32, event::KeyCode::Colon as u32);
        assert_eq!(VirtualKeyCode::Comma as u32, event::KeyCode::Comma as u32);
        assert_eq!(
            VirtualKeyCode::Convert as u32,
            event::KeyCode::Convert as u32
        );
        assert_eq!(VirtualKeyCode::Equals as u32, event::KeyCode::Equals as u32);
        assert_eq!(VirtualKeyCode::Grave as u32, event::KeyCode::Grave as u32);
        assert_eq!(VirtualKeyCode::Kana as u32, event::KeyCode::Kana as u32);
        assert_eq!(VirtualKeyCode::Kanji as u32, event::KeyCode::Kanji as u32);
        assert_eq!(VirtualKeyCode::LAlt as u32, event::KeyCode::LAlt as u32);
        assert_eq!(
            VirtualKeyCode::LBracket as u32,
            event::KeyCode::LBracket as u32
        );
        assert_eq!(
            VirtualKeyCode::LControl as u32,
            event::KeyCode::LControl as u32
        );
        assert_eq!(VirtualKeyCode::LShift as u32, event::KeyCode::LShift as u32);
        assert_eq!(VirtualKeyCode::LWin as u32, event::KeyCode::LWin as u32);
        assert_eq!(VirtualKeyCode::Mail as u32, event::KeyCode::Mail as u32);
        assert_eq!(
            VirtualKeyCode::MediaSelect as u32,
            event::KeyCode::MediaSelect as u32
        );
        assert_eq!(
            VirtualKeyCode::MediaStop as u32,
            event::KeyCode::MediaStop as u32
        );
        assert_eq!(VirtualKeyCode::Minus as u32, event::KeyCode::Minus as u32);
        assert_eq!(VirtualKeyCode::Mute as u32, event::KeyCode::Mute as u32);
        assert_eq!(
            VirtualKeyCode::MyComputer as u32,
            event::KeyCode::MyComputer as u32
        );
        assert_eq!(
            VirtualKeyCode::NavigateForward as u32,
            event::KeyCode::NavigateForward as u32
        );
        assert_eq!(
            VirtualKeyCode::NavigateBackward as u32,
            event::KeyCode::NavigateBackward as u32
        );
        assert_eq!(
            VirtualKeyCode::NextTrack as u32,
            event::KeyCode::NextTrack as u32
        );
        assert_eq!(
            VirtualKeyCode::NoConvert as u32,
            event::KeyCode::NoConvert as u32
        );
        assert_eq!(VirtualKeyCode::OEM102 as u32, event::KeyCode::OEM102 as u32);
        assert_eq!(VirtualKeyCode::Period as u32, event::KeyCode::Period as u32);
        assert_eq!(
            VirtualKeyCode::PlayPause as u32,
            event::KeyCode::PlayPause as u32
        );
        assert_eq!(VirtualKeyCode::Plus as u32, event::KeyCode::Plus as u32);
        assert_eq!(VirtualKeyCode::Power as u32, event::KeyCode::Power as u32);
        assert_eq!(
            VirtualKeyCode::PrevTrack as u32,
            event::KeyCode::PrevTrack as u32
        );
        assert_eq!(VirtualKeyCode::RAlt as u32, event::KeyCode::RAlt as u32);
        assert_eq!(
            VirtualKeyCode::RBracket as u32,
            event::KeyCode::RBracket as u32
        );
        assert_eq!(
            VirtualKeyCode::RControl as u32,
            event::KeyCode::RControl as u32
        );
        assert_eq!(VirtualKeyCode::RShift as u32, event::KeyCode::RShift as u32);
        assert_eq!(VirtualKeyCode::RWin as u32, event::KeyCode::RWin as u32);
        assert_eq!(
            VirtualKeyCode::Semicolon as u32,
            event::KeyCode::Semicolon as u32
        );
        assert_eq!(VirtualKeyCode::Slash as u32, event::KeyCode::Slash as u32);
        assert_eq!(VirtualKeyCode::Sleep as u32, event::KeyCode::Sleep as u32);
        assert_eq!(VirtualKeyCode::Stop as u32, event::KeyCode::Stop as u32);
        assert_eq!(VirtualKeyCode::Sysrq as u32, event::KeyCode::Sysrq as u32);
        assert_eq!(VirtualKeyCode::Tab as u32, event::KeyCode::Tab as u32);
        assert_eq!(
            VirtualKeyCode::Underline as u32,
            event::KeyCode::Underline as u32
        );
        assert_eq!(
            VirtualKeyCode::Unlabeled as u32,
            event::KeyCode::Unlabeled as u32
        );
        assert_eq!(
            VirtualKeyCode::VolumeDown as u32,
            event::KeyCode::VolumeDown as u32
        );
        assert_eq!(
            VirtualKeyCode::VolumeUp as u32,
            event::KeyCode::VolumeUp as u32
        );
        assert_eq!(VirtualKeyCode::Wake as u32, event::KeyCode::Wake as u32);
        assert_eq!(
            VirtualKeyCode::WebBack as u32,
            event::KeyCode::WebBack as u32
        );
        assert_eq!(
            VirtualKeyCode::WebFavorites as u32,
            event::KeyCode::WebFavorites as u32
        );
        assert_eq!(
            VirtualKeyCode::WebForward as u32,
            event::KeyCode::WebForward as u32
        );
        assert_eq!(
            VirtualKeyCode::WebHome as u32,
            event::KeyCode::WebHome as u32
        );
        assert_eq!(
            VirtualKeyCode::WebRefresh as u32,
            event::KeyCode::WebRefresh as u32
        );
        assert_eq!(
            VirtualKeyCode::WebSearch as u32,
            event::KeyCode::WebSearch as u32
        );
        assert_eq!(
            VirtualKeyCode::WebStop as u32,
            event::KeyCode::WebStop as u32
        );
        assert_eq!(VirtualKeyCode::Yen as u32, event::KeyCode::Yen as u32);
        assert_eq!(VirtualKeyCode::Copy as u32, event::KeyCode::Copy as u32);
        assert_eq!(VirtualKeyCode::Paste as u32, event::KeyCode::Paste as u32);
        assert_eq!(VirtualKeyCode::Cut as u32, event::KeyCode::Cut as u32);
    }
}
