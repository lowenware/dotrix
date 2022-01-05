use dotrix::input::{Button, State as InputState};
use dotrix::prelude::*;
use dotrix::{Camera, Frame, Input, Window};

/// This func updates camera based on controls
pub fn camera_update(
    camera: Mut<Camera>,
    frame: Const<Frame>,
    input: Const<Input>,
    mut window: Mut<Window>,
) {
    if let Some(state) = input.button_state(Button::MouseRight) {
        match state {
            InputState::Activated => {
                window.set_cursor_visible(false);
            }
            InputState::Hold => {
                dotrix::camera::control(camera, input, frame);
            }
            InputState::Deactivated => window.set_cursor_visible(true),
        }
    }
}
