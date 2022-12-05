mod event;

use dotrix_core as dotrix;
use dotrix_types::Frame;

pub use event::{Button, Event, KeyCode, Modifiers, MouseScroll, ScanCode};

/// Inputs for the current frame
pub struct Input {
    pub events: Vec<Event>,
    pub modifiers: Modifiers,
}

impl Input {
    pub fn text(&self) -> String {
        self.events
            .iter()
            .map(|e| match e {
                Event::CharacterInput { character } => {
                    let chr = *character;
                    if is_printable(chr) {
                        Some(chr)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .filter(|i| i.is_some())
            .map(|i| i.unwrap())
            .collect::<String>()
    }
}

#[derive(Default)]
pub struct ListenTask {
    modifiers: Modifiers,
}

impl ListenTask {
    pub fn new() -> Self {
        Self::default()
    }
}

impl dotrix::Task for ListenTask {
    type Context = (dotrix::Collect<Event>, dotrix::Any<Frame>);
    type Output = Input; // :)
    fn run(&mut self, (events, _): Self::Context) -> Self::Output {
        let events = events.collect();
        for event in events.iter() {
            if let Event::ModifiersChange { modifiers } = event {
                self.modifiers = *modifiers;
            }
        }

        Input {
            events,
            modifiers: self.modifiers,
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
