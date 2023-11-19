//! Camera module

use std::ops::Range;

use dotrix_math::{perspective, Mat3, Mat4, Point3, Quat, Rad, Vec3};

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
    pub fn lens(fov: impl Into<Rad<f32>>, plane: Range<f32>) -> Lens {
        Lens::new(fov, plane)
    }
}

/// Projection matrix constructor
pub struct Lens {
    /// Field of View (rad)
    pub fov: Rad<f32>,
    /// Near..Far plane
    pub plane: Range<f32>,
}

impl Lens {
    /// Returns new instance of projection matrix constructor
    pub fn new(fov: impl Into<Rad<f32>>, plane: Range<f32>) -> Self {
        Self {
            fov: fov.into(),
            plane,
        }
    }

    /// Returns projection matrix for the surface
    pub fn proj(&self, surface_width: u32, surface_height: u32) -> Mat4 {
        let aspect_ratio = surface_width as f32 / surface_height as f32;
        perspective(self.fov, aspect_ratio, self.plane.start, self.plane.end)
    }
}

impl Default for Lens {
    fn default() -> Self {
        Self {
            fov: Rad(1.1),
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
        let rx = Mat3::from_angle_x(Rad(roll));
        let ry = Mat3::from_angle_y(Rad(pitch));
        let rz = Mat3::from_angle_z(Rad(yaw));

        let mut mx = Mat4::from(rx * ry * rz);
        mx.w.x = self.point.x;
        mx.w.y = self.point.y;
        mx.w.z = self.point.z;

        mx
    }

    /// Return view matrix made from target
    pub fn target(&self, target: Vec3) -> Mat4 {
        self.target_up(target, Vec3::unit_z())
    }

    /// Return view matrix made from target and up vector
    pub fn target_up(&self, target: Vec3, up: Vec3) -> Mat4 {
        Mat4::look_at_rh(
            Point3::new(self.point.x, self.point.y, self.point.z),
            Point3::new(target.x, target.y, target.z),
            up,
        )
    }

    /// Return view matrix for camera flying around a target (self.point)
    pub fn follow(self, distance: f32, pan: f32, tilt: f32, roll: f32) -> Mat4 {
        use dotrix_math::{InnerSpace, Rotation3};

        let target = &self.point;
        let dz = distance * tilt.sin();
        let dxy = distance * tilt.cos();
        let dx = dxy * pan.cos();
        let dy = dxy * pan.sin();
        let position = Vec3::new(target.x + dx, target.y + dy, target.z + dz);
        let direction = (target - position).normalize();
        let roll = Quat::from_axis_angle(direction, Rad(roll));
        let camera_right = direction.cross(Vec3::unit_z());
        let camera_up = roll * camera_right.cross(direction);

        Mat4::look_at_rh(
            Point3::new(position.x, position.y, position.z),
            Point3::new(target.x, target.y, target.z),
            camera_up,
        )
    }
}
