use dotrix::camera::Camera;
use dotrix::log;
use dotrix::math::{Mat4, Rad, Vec3};
use dotrix_input::Button;

pub struct ControlTask {
    proj: Option<Mat4>,
    target: Vec3,
    distance: f32,
    pan: f32,
    tilt: f32,
}

impl ControlTask {
    pub fn new() -> Self {
        let target = Vec3::new(0.0, 0.0, 0.0);
        Self {
            proj: None,
            target,
            distance: 30.0,
            pan: 0.0 * std::f32::consts::PI / 180.0,
            tilt: 30.0 * std::f32::consts::PI / 180.0,
        }
    }
}

impl dotrix::Task for ControlTask {
    type Context = (
        dotrix::Any<dotrix::Frame>,
        dotrix::Any<dotrix::Input>,
        dotrix::All<dotrix::ui::Overlay>,
    );

    type Output = Camera;

    fn run(&mut self, (frame, input, _): Self::Context) -> Self::Output {
        const ROTATE_SPEED: f32 = std::f32::consts::PI / 5.0;
        const ZOOM_SPEED: f32 = 130.0;
        let time_delta = frame.delta.as_secs_f32();
        let mouse_delta = input.mouse_move_delta;

        // camera distace
        self.distance -= (ZOOM_SPEED * input.mouse_scroll_delta_lines.vertical as f32
            + input.mouse_scroll_delta_pixels.vertical as f32)
            * time_delta;

        if frame.resized {
            self.proj.take();
        }

        if input.hold.get(&Button::MouseRight).is_some() {
            // angle around the target
            self.pan -= mouse_delta.horizontal as f32 * ROTATE_SPEED * time_delta;
            // angle above the target
            let tilt = self.tilt + mouse_delta.vertical as f32 * ROTATE_SPEED * time_delta;
            self.tilt = tilt.clamp(
                -std::f32::consts::PI / 2.0 + 0.001,
                std::f32::consts::PI / 2.0 - 0.001,
            );
            log::debug!("Vertical: {}, {}", mouse_delta.vertical, self.tilt);
        }
        let roll = 0.0;

        // matrices
        let view = Camera::at(self.target.x, self.target.y, self.target.z).follow(
            self.distance,
            self.pan,
            self.tilt,
            roll,
        );

        let proj = self
            .proj
            .get_or_insert_with(|| {
                Camera::lens(Rad(1.1), 0.0625..524288.06).proj(frame.width, frame.height)
            })
            .clone();

        Camera::new(proj, view)
    }
}
