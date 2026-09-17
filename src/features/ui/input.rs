use crate::window::event::{Button, Event};
use crate::{Any, Input, Mut, Task};

use super::Overlay;

use super::layout::LayoutRect;

/// Interaction state from the previous frame (read during build, written during input).
#[derive(Debug, Default, Clone)]
pub struct UiState {
    pub hovered_region: Option<usize>,
    pub pressed_region: Option<usize>,
    pub clicked_region: Option<usize>,
}

/// Hit-test target appended during UI build.
#[derive(Debug, Clone, Copy)]
pub struct HitRegion {
    pub index: usize,
    pub rect: LayoutRect,
    pub kind: HitKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    Panel,
    Button,
}

/// Marker output so [`ProcessUiInput`] does not share the `()` output channel with other tasks.
#[derive(Debug, Default, Clone, Copy)]
pub struct UiInputFrame;

/// Hit-tests the current frame overlay and updates [`UiState`].
#[derive(Default)]
pub struct ProcessUiInput;

impl Task for ProcessUiInput {
    type Context = (Any<Overlay>, Any<Input>, Mut<UiState>);
    type Output = UiInputFrame;

    fn run(&mut self, (overlay, input, mut ui_state): Self::Context) -> Self::Output {
        let mx = input.mouse_position.horizontal as f32;
        let my = input.mouse_position.vertical as f32;

        let hovered = overlay
            .hit_regions
            .iter()
            .rev()
            .find(|region| region.rect.contains(mx, my))
            .map(|region| region.index);

        let left_down = input.hold.contains_key(&Button::MouseLeft);
        let left_released = input.events.iter().any(|event| {
            matches!(
                event,
                Event::ButtonRelease {
                    button: Button::MouseLeft
                }
            )
        });

        // Count a click when the button that was pressed is released (even if the
        // cursor moved slightly off the widget before release).
        let clicked = if left_released {
            ui_state.pressed_region
        } else {
            None
        };

        ui_state.hovered_region = hovered;
        ui_state.pressed_region = if left_down { hovered } else { None };
        ui_state.clicked_region = clicked;
        UiInputFrame
    }
}
