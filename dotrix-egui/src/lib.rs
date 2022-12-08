use std::ops::Deref;

use dotrix_core as dotrix;
use dotrix_input::{
    Button, DragAndDrop, Event, Input, KeyCode, Modifiers, MouseScroll, ScreenVector,
};
use dotrix_types::Frame;

pub use egui as backend;

pub struct Context {
    ctx: egui::Context,
}

impl Deref for Context {
    type Target = egui::Context;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

struct LoadTask {
    ctx: egui::Context,
}

impl dotrix::Task for LoadTask {
    type Context = (dotrix::Any<Frame>, dotrix::Any<Input>);
    type Output = Context;

    fn run(&mut self, (frame, input): Self::Context) -> Self::Output {
        let scale_factor = frame.scale_factor;
        let screen_rect = Some(egui::Rect::from_min_size(
            Default::default(),
            egui::vec2(frame.width as f32, frame.height as f32) / scale_factor,
        ));

        let mut dropped_files = vec![];
        let mut hovered_files = vec![];

        let mut current_modifiers = input.modifiers.clone();
        let mut mouse_position = input.mouse_position;

        let wants_keyboard_input = self.ctx.wants_keyboard_input();
        let wants_pointer_input = self.ctx.wants_pointer_input();

        let mut events = input
            .events
            .iter()
            .flat_map(|e| match e {
                Event::ModifiersChange { modifiers } => {
                    current_modifiers = modifiers.clone();
                    None
                }
                Event::CursorPosition {
                    horizontal,
                    vertical,
                } => {
                    mouse_position.horizontal = *horizontal;
                    mouse_position.vertical = *vertical;
                    None
                }
                Event::ButtonPress { button } => to_egui_button_event(
                    button,
                    true,
                    current_modifiers,
                    mouse_position,
                    wants_keyboard_input,
                    wants_pointer_input,
                ),
                Event::ButtonRelease { button } => to_egui_button_event(
                    button,
                    false,
                    current_modifiers,
                    mouse_position,
                    wants_keyboard_input,
                    wants_pointer_input,
                ),
                Event::MouseScroll { delta } => {
                    let (horizontal, vertical) = match delta {
                        MouseScroll::Lines {
                            horizontal,
                            vertical,
                        } => (*horizontal, *vertical),
                        MouseScroll::Pixels {
                            horizontal,
                            vertical,
                        } => (*horizontal / 12.0, *vertical / 12.0),
                    };
                    Some(egui::Event::Scroll(egui::vec2(
                        horizontal as f32,
                        vertical as f32,
                    )))
                }
                Event::DragAndDrop { target } => {
                    match target {
                        DragAndDrop::FileDragged { path } => {
                            hovered_files.push(egui::HoveredFile {
                                path: Some(path.to_owned()),
                                ..Default::default()
                            })
                        }
                        DragAndDrop::FileDropped { path } => {
                            dropped_files.push(egui::DroppedFile {
                                path: Some(path.to_owned()),
                                ..Default::default()
                            })
                        }
                        DragAndDrop::Canceled => {
                            hovered_files.clear();
                        }
                    };
                    None
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        if input.text.len() > 0 {
            events.push(egui::Event::Text(input.text.clone()));
        }

        self.ctx.begin_frame(egui::RawInput {
            screen_rect,
            pixels_per_point: Some(scale_factor),
            events,
            dropped_files,
            hovered_files,
            ..Default::default()
        });

        Context {
            ctx: self.ctx.clone(),
        }
    }
}

#[inline]
fn to_egui_button_event(
    button: &Button,
    pressed: bool,
    modifiers: Modifiers,
    mouse_position: ScreenVector,
    wants_keyboard_input: bool,
    wants_pointer_input: bool,
) -> Option<egui::Event> {
    let button = match button {
        Button::MouseLeft => egui::PointerButton::Primary,
        Button::MouseRight => egui::PointerButton::Secondary,
        Button::MouseMiddle => egui::PointerButton::Middle,
        Button::Key { key_code, .. } => {
            return (if wants_keyboard_input {
                key_code
            } else {
                &None
            })
            .and_then(|key_code| {
                Some(match key_code {
                    KeyCode::Left => egui::Key::ArrowLeft,
                    KeyCode::Right => egui::Key::ArrowRight,
                    KeyCode::Up => egui::Key::ArrowUp,
                    KeyCode::Escape => egui::Key::Escape,
                    KeyCode::Tab => egui::Key::Tab,
                    KeyCode::Backspace => egui::Key::Backspace,
                    KeyCode::Return => egui::Key::Enter,
                    KeyCode::Space => egui::Key::Space,
                    KeyCode::Insert => egui::Key::Insert,
                    KeyCode::Delete => egui::Key::Delete,
                    KeyCode::Home => egui::Key::Home,
                    KeyCode::End => egui::Key::End,
                    KeyCode::PageUp => egui::Key::PageUp,
                    KeyCode::PageDown => egui::Key::PageDown,
                    KeyCode::Minus => egui::Key::Minus,
                    KeyCode::Equals => egui::Key::PlusEquals,
                    KeyCode::Key0 => egui::Key::Num0,
                    KeyCode::Key1 => egui::Key::Num1,
                    KeyCode::Key2 => egui::Key::Num2,
                    KeyCode::Key3 => egui::Key::Num3,
                    KeyCode::Key4 => egui::Key::Num4,
                    KeyCode::Key5 => egui::Key::Num5,
                    KeyCode::Key6 => egui::Key::Num6,
                    KeyCode::Key7 => egui::Key::Num7,
                    KeyCode::Key8 => egui::Key::Num8,
                    KeyCode::Key9 => egui::Key::Num9,
                    KeyCode::A => egui::Key::A,
                    KeyCode::B => egui::Key::B,
                    KeyCode::C => egui::Key::C,
                    KeyCode::D => egui::Key::D,
                    KeyCode::E => egui::Key::E,
                    KeyCode::F => egui::Key::F,
                    KeyCode::G => egui::Key::G,
                    KeyCode::H => egui::Key::H,
                    KeyCode::I => egui::Key::I,
                    KeyCode::J => egui::Key::J,
                    KeyCode::K => egui::Key::K,
                    KeyCode::L => egui::Key::L,
                    KeyCode::M => egui::Key::M,
                    KeyCode::N => egui::Key::N,
                    KeyCode::O => egui::Key::O,
                    KeyCode::P => egui::Key::P,
                    KeyCode::Q => egui::Key::Q,
                    KeyCode::R => egui::Key::R,
                    KeyCode::S => egui::Key::S,
                    KeyCode::T => egui::Key::T,
                    KeyCode::U => egui::Key::U,
                    KeyCode::V => egui::Key::V,
                    KeyCode::W => egui::Key::W,
                    KeyCode::X => egui::Key::X,
                    KeyCode::Y => egui::Key::Y,
                    KeyCode::Z => egui::Key::Z,
                    KeyCode::F1 => egui::Key::F1,
                    KeyCode::F2 => egui::Key::F2,
                    KeyCode::F3 => egui::Key::F3,
                    KeyCode::F4 => egui::Key::F4,
                    KeyCode::F5 => egui::Key::F5,
                    KeyCode::F6 => egui::Key::F6,
                    KeyCode::F7 => egui::Key::F7,
                    KeyCode::F8 => egui::Key::F8,
                    KeyCode::F9 => egui::Key::F9,
                    KeyCode::F10 => egui::Key::F10,
                    KeyCode::F11 => egui::Key::F11,
                    KeyCode::F12 => egui::Key::F12,
                    KeyCode::F13 => egui::Key::F13,
                    KeyCode::F14 => egui::Key::F14,
                    KeyCode::F15 => egui::Key::F15,
                    KeyCode::F16 => egui::Key::F16,
                    KeyCode::F17 => egui::Key::F17,
                    KeyCode::F18 => egui::Key::F18,
                    KeyCode::F19 => egui::Key::F19,
                    KeyCode::F20 => egui::Key::F20,
                    _ => return None,
                })
            })
            .and_then(|key| {
                Some(egui::Event::Key {
                    key,
                    pressed,
                    modifiers: to_egui_modifiers(modifiers),
                })
            });
        }
        _ => return None,
    };

    if wants_pointer_input {
        return Some(egui::Event::PointerButton {
            pos: egui::pos2(
                mouse_position.horizontal as f32,
                mouse_position.vertical as f32,
            ),
            button,
            pressed,
            modifiers: to_egui_modifiers(modifiers),
        });
    }

    None
}

/// Translates dotrix to egui modifier keys.
#[inline]
fn to_egui_modifiers(modifiers: Modifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.ctrl(),
        shift: modifiers.shift(),
        #[cfg(target_os = "macos")]
        mac_cmd: modifiers.cmd(),
        #[cfg(target_os = "macos")]
        command: modifiers.cmd(),
        #[cfg(not(target_os = "macos"))]
        mac_cmd: false,
        #[cfg(not(target_os = "macos"))]
        command: modifiers.ctrl(),
    }
}
