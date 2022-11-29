use dotrix::camera::{Camera, Lens};
use dotrix::math::Vec3;

pub struct ControlTask {
    position: Vec3,
    target: Vec3,
}

impl ControlTask {
    pub fn new() -> Self {
        let position = Vec3::new(20.0, -30.0, 20.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let lens = Lens::default();
        Self { position, target }
    }
}

impl dotrix::Task for ControlTask {
    type Context = (dotrix::Any<dotrix::gpu::Frame>,);

    type Output = Camera;

    fn run(&mut self, (frame,): Self::Context) -> Self::Output {
        let view = Camera::at(self.position.x, self.position.y, self.position.z)
            .target(self.target, Vec3::unit_z());

        let proj =
            Camera::lens(1.1, 0.0625..524288.06).matrix(frame.surface_width, frame.surface_height);

        Camera::new(proj, view)
    }
}
