//! Camera module

use std::ops::Range;

use crate::math::{Mat3, Mat4, Quat, Vec3};

/// Camera object and constructor
pub struct Camera {
    /// Projection matrix
    pub proj: Mat4,
    /// View matrix
    pub view: Mat4,
}

impl Camera {
    /// Constructs new instance of Camera
    pub fn new(proj: Mat4, view: Mat4) -> Self {
        Self { proj, view }
    }

    /// Returns view matrix constructor
    pub fn at(x: f32, y: f32, z: f32) -> View {
        View::new(x, y, z)
    }

    /// Returns projection matrix constructor
    pub fn lens(fov: impl Into<f32>, plane: Range<f32>) -> Lens {
        Lens::new(fov, plane)
    }
}

/// Projection matrix constructor
pub struct Lens {
    /// Field of View (rad)
    pub fov: f32,
    /// Near..Far plane
    pub plane: Range<f32>,
}

impl Lens {
    /// Returns new instance of projection matrix constructor
    pub fn new(fov: impl Into<f32>, plane: Range<f32>) -> Self {
        Self {
            fov: fov.into(),
            plane,
        }
    }

    /// Returns projection matrix for the surface
    pub fn proj(&self, surface_width: u32, surface_height: u32) -> Mat4 {
        let aspect_ratio = surface_width as f32 / surface_height as f32;
        Mat4::perspective_rh(self.fov, aspect_ratio, self.plane.start, self.plane.end)
    }
}

impl Default for Lens {
    fn default() -> Self {
        Self {
            fov: 1.1, // std::f32::consts::FRAC_PI_4
            plane: 0.0625..524288.06,
        }
    }
}

/// View matrix constructor
pub struct View {
    /// Either camera position or target
    pub point: Vec3,
}

impl View {
    /// Returns new view matrix constructor
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            point: Vec3::new(x, y, z),
        }
    }

    /// Return view matrix made from rotations
    ///
    /// self.point is handled as camera position
    pub fn rotate(self, pitch: f32, yaw: f32, roll: f32) -> Mat4 {
        let rx = Mat3::from_rotation_x(roll);
        let ry = Mat3::from_rotation_y(pitch);
        let rz = Mat3::from_rotation_z(yaw);

        let mut mx = Mat4::from_mat3(rx * ry * rz);
        mx.w_axis.x = self.point.x;
        mx.w_axis.y = self.point.y;
        mx.w_axis.z = self.point.z;

        mx
    }

    /// Return view matrix made from target
    pub fn target(&self, target: Vec3) -> Mat4 {
        self.target_up(target, Vec3::Z)
    }

    /// Return view matrix made from target and up vector
    pub fn target_up(&self, target: Vec3, up: Vec3) -> Mat4 {
        // let view = Mat4::look_at_rh(Vec3::new(1.5f32, -5.0, 3.0), Vec3::ZERO, Vec3::Z);
        Mat4::look_at_rh(
            Vec3::new(self.point.x, self.point.y, self.point.z),
            Vec3::new(target.x, target.y, target.z),
            up,
        )
    }

    /// Return view matrix for camera flying around a target (self.point)
    pub fn follow(self, distance: f32, pan: f32, tilt: f32, roll: f32) -> Mat4 {
        let target = self.point;
        let dz = distance * tilt.sin();
        let dxy = distance * tilt.cos();
        let dx = dxy * pan.cos();
        let dy = dxy * pan.sin();
        let position = Vec3::new(target.x + dx, target.y + dy, target.z + dz);
        let direction = (target - position).normalize();
        let roll = Quat::from_axis_angle(direction, roll);
        let camera_right = direction.cross(Vec3::Z);
        let camera_up = roll * camera_right.cross(direction);

        Mat4::look_at_rh(
            Vec3::new(position.x, position.y, position.z),
            Vec3::new(target.x, target.y, target.z),
            camera_up,
        )
    }
}
