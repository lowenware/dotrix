mod event;

use std::collections::HashMap;
use std::time::Instant;

use dotrix_core as dotrix;
use dotrix_log as log;
use dotrix_types::Frame;

pub use event::{Button, DragAndDrop, Event, KeyCode, Modifiers, MouseScroll, ScanCode};

#[derive(Debug, Default, Clone, Copy)]
pub struct ScreenVector {
    pub horizontal: f64,
    pub vertical: f64,
}

/// Inputs for the current frame
#[derive(Debug)]
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
pub struct ListenTask {
    modifiers: Modifiers,
    hold: HashMap<Button, Instant>,
    mouse_position: ScreenVector,
}

impl ListenTask {
    pub fn new() -> Self {
        Self::default()
    }
}

impl dotrix::Task for ListenTask {
    type Context = (dotrix::Take<dotrix::All<Event>>, dotrix::Any<Frame>);
    type Output = Input; // :)
    fn run(&mut self, (events, _): Self::Context) -> Self::Output {
        log::debug!("Input:");
        let events = events.take();
        log::debug!("{:?}", events);

        let mut text = String::with_capacity(8);
        let mut mouse_move_delta = ScreenVector::default();
        let mut mouse_scroll_delta_lines = ScreenVector::default();
        let mut mouse_scroll_delta_pixels = ScreenVector::default();
        let mut mouse_position = self.mouse_position.clone();

        for event in events.iter() {
            match event {
                Event::ModifiersChange { modifiers } => {
                    self.modifiers = *modifiers;
                }
                Event::ButtonPress { button } => {
                    self.hold.entry(*button).or_insert_with(|| Instant::now());
                }
                Event::ButtonRelease { button } => {
                    self.hold.remove(&button);
                }
                Event::MouseMove {
                    horizontal,
                    vertical,
                } => {
                    mouse_move_delta.horizontal += *horizontal;
                    mouse_move_delta.vertical += *vertical;
                }
                Event::MouseScroll { delta } => match delta {
                    MouseScroll::Lines {
                        horizontal,
                        vertical,
                    } => {
                        mouse_scroll_delta_lines.horizontal += *horizontal;
                        mouse_scroll_delta_lines.vertical += *vertical;
                    }
                    MouseScroll::Pixels {
                        horizontal,
                        vertical,
                    } => {
                        mouse_scroll_delta_pixels.horizontal += *horizontal;
                        mouse_scroll_delta_pixels.vertical += *vertical;
                    }
                },
                Event::CursorPosition {
                    horizontal,
                    vertical,
                } => {
                    mouse_position.horizontal = *horizontal;
                    mouse_position.vertical = *vertical;
                }
                Event::CharacterInput { character } => {
                    let chr = *character;
                    if is_printable(chr) {
                        text.push(chr);
                    }
                }
                _ => {}
            }
        }

        Input {
            events,
            modifiers: self.modifiers,
            hold: self.hold.clone(),
            mouse_position,
            mouse_move_delta,
            mouse_scroll_delta_lines,
            mouse_scroll_delta_pixels,
            text,
        }
    }
}

#[inline]
fn is_printable(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}
