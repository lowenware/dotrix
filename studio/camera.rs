use dotrix::camera::Camera;
use dotrix::log;
use dotrix::math::{Rad, Vec3};

pub struct ControlTask {
    position: Vec3,
    target: Vec3,
}

impl ControlTask {
    pub fn new() -> Self {
        let position = Vec3::new(20.0, -30.0, 20.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        Self { position, target }
    }
}

impl dotrix::Task for ControlTask {
    type Context = (dotrix::Any<dotrix::Frame>, dotrix::All<dotrix::ui::Overlay>);

    type Output = Camera;

    fn run(&mut self, (frame, _): Self::Context) -> Self::Output {
        let view =
            Camera::at(self.position.x, self.position.y, self.position.z).target(self.target);

        let proj = Camera::lens(Rad(1.1), 0.0625..524288.06).proj(frame.width, frame.height);

        Camera::new(proj, view)
    }
}
