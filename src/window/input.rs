use std::collections::HashMap;
use std::time::Instant;

use crate::graphics::Frame;
use crate::tasks::{All, Any, Take, Task};

pub use super::event::{
    Button,
    // DragAndDrop,
    Event,
    // KeyCode,
    Modifiers,
    MouseScroll,
    // ScanCode
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ScreenVector {
    pub horizontal: f64,
    pub vertical: f64,
}

/// Inputs for the current frame
#[derive(Debug, Default, Clone)]
pub struct Input {
    pub events: Vec<Event>,
    pub modifiers: Modifiers,
    pub text: String,
    pub hold: HashMap<Button, Instant>,
    pub mouse_position: ScreenVector,
    pub mouse_move_delta: ScreenVector,
    pub mouse_scroll_delta_lines: ScreenVector,
    pub mouse_scroll_delta_pixels: ScreenVector,
}

#[derive(Default)]
pub struct ReadInput {
    modifiers: Modifiers,
    hold: HashMap<Button, Instant>,
    mouse_position: ScreenVector,
}

impl ReadInput {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Task for ReadInput {
    type Context = (Take<All<Event>>, Any<Frame>);
    type Output = Input; // :)
    fn run(&mut self, (mut events, _): Self::Context) -> Self::Output {
        let capacity = events.len();
        let mut list = Vec::with_capacity(capacity);

        let mut input_text = String::with_capacity(8);
        let mut mouse_move_delta = ScreenVector::default();
        let mut mouse_scroll_delta_lines = ScreenVector::default();
        let mut mouse_scroll_delta_pixels = ScreenVector::default();
        let mut mouse_position = self.mouse_position;

        for event in events.drain() {
            match &event {
                Event::ModifiersChange { modifiers } => {
                    self.modifiers = *modifiers;
                }
                Event::ButtonPress { button, text } => {
                    if let Some(text) = text.as_ref() {
                        input_text += text;
                    }
                    self.hold.entry(*button).or_insert_with(Instant::now);
                }
                Event::ButtonRelease { button } => {
                    self.hold.remove(button);
                }
                Event::MouseMove {
                    horizontal,
                    vertical,
                } => {
                    mouse_move_delta.horizontal += horizontal;
                    mouse_move_delta.vertical += vertical;
                }
                Event::MouseScroll { delta } => match delta {
                    MouseScroll::Lines {
                        horizontal,
                        vertical,
                    } => {
                        mouse_scroll_delta_lines.horizontal += *horizontal as f64;
                        mouse_scroll_delta_lines.vertical += *vertical as f64;
                    }
                    MouseScroll::Pixels {
                        horizontal,
                        vertical,
                    } => {
                        mouse_scroll_delta_pixels.horizontal += horizontal;
                        mouse_scroll_delta_pixels.vertical += vertical;
                    }
                },
                Event::CursorPosition {
                    horizontal,
                    vertical,
                } => {
                    mouse_position.horizontal = *horizontal;
                    mouse_position.vertical = *vertical;
                }
                _ => {}
            }
            list.push(event);
        }

        Input {
            events: list,
            modifiers: self.modifiers,
            hold: self.hold.clone(),
            mouse_position,
            mouse_move_delta,
            mouse_scroll_delta_lines,
            mouse_scroll_delta_pixels,
            text: input_text,
        }
    }
}

/*
#[inline]
fn is_printable(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}
*/
