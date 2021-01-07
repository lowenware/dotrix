use crate::{
    ecs::{ Mut, Const },
    services::{ Input, Frame },
};

use dotrix_math::{Mat4, Point3, Vec3};
use std::f32::consts::PI;

const ROTATE_SPEED: f32 = PI / 10.0;
const ZOOM_SPEED: f32 = 10.0;

pub struct Camera {
    pub distance: f32,
    pub y_angle: f32,
    pub xz_angle: f32,
    pub target: Point3,
    pub view: Option<Mat4>,
}

impl Camera {
    pub fn new() -> Self {
        let distance = 15.0;
        let y_angle = -PI / 2.0;
        let xz_angle = PI / 4.0;
        let target = Point3::new(0.0, 5.0, 0.0);

        Self {
            distance,
            y_angle,
            xz_angle,
            target,
            view: None,
        }
    }

    pub fn view(&self) -> Mat4 {
        self.view
            .as_ref()
            .copied()
            .unwrap_or_else(|| {
                Self::matrix(self.target, self.distance, self.y_angle, self.xz_angle)
            })
    }

    pub fn view_static(&self) -> Mat4 {
        let mut view_static = self.view();
        view_static.w.x = 0.0;
        view_static.w.y = 0.0;
        view_static.w.z = 0.0;
        view_static
    }

    pub fn set_view(&mut self) {
        self.view = Some(Self::matrix(self.target, self.distance, self.y_angle, self.xz_angle));
    }

    pub fn position(&self) -> Vec3 {
        let dy = self.distance * self.xz_angle.sin();
        let dxz = self.distance * self.xz_angle.cos();
        let dx = dxz * self.y_angle.cos();
        let dz = dxz * self.y_angle.sin();
        Vec3::new(self.target.x + dx, self.target.y + dy, self.target.z + dz)
    }

    fn matrix(target: Point3, distance: f32, y_angle: f32, xz_angle: f32) -> Mat4 {
        let dy = distance * xz_angle.sin();
        let dxz = distance * xz_angle.cos();
        let dx = dxz * y_angle.cos();
        let dz = dxz * y_angle.sin();
        let position = Point3::new(target.x + dx, target.y + dy, target.z + dz);

        let (position, target) = if distance > 0.0 {
            (position, target)
        } else {
            (target, position)
        };

        Mat4::look_at(position, target, Vec3::unit_y())
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

pub fn camera_control(mut camera: Mut<Camera>, input: Const<Input>, frame: Const<Frame>) {
    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    let distance = camera.distance - ZOOM_SPEED * mouse_scroll * time_delta;
    camera.distance = if distance > -1.0 { distance } else { -1.0 };

    camera.y_angle += mouse_delta.x * ROTATE_SPEED * time_delta;

    let xz_angle = camera.xz_angle + mouse_delta.y * ROTATE_SPEED * time_delta;  
    let half_pi = PI / 2.0;

    camera.xz_angle = if xz_angle >= half_pi {
        half_pi - 0.01
    } else if xz_angle <= -half_pi {
        -half_pi + 0.01
    } else {
        xz_angle
    };

    camera.set_view();
}
