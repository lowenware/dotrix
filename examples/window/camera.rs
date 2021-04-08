use dotrix::{
    ecs::{ Const, Mut },
    input::{ Button, State as InputState },
    math::{ Vec2 },
    services::{ Camera, Frame, Input, Window },
    systems::camera_control,
};

pub struct CameraMemory {
    cursor_position: Vec2,
}

impl CameraMemory {
    pub fn new() -> Self {
        Self {
            cursor_position: Vec2::new(0.0, 0.0)
        }
    }
}

/// This func updates camera based on controls
pub fn camera_update(
    camera: Mut<Camera>,
    frame: Const<Frame>,
    input: Const<Input>,
    mut memory: Mut<CameraMemory>,
    mut window: Mut<Window>,
) {

    if let Some(state) = input.button_state(Button::MouseRight) {
        match state {
            InputState::Activated => {
                memory.cursor_position = *input.mouse_position().unwrap(); // TODO: why option?
                window.set_cursor_visible(false);
            },
            InputState::Hold => {
                camera_control(camera, input, frame);
            },
            InputState::Deactivated => {
                window.set_cursor_position(memory.cursor_position);
                window.set_cursor_visible(true)
            },
        }
    }
}
