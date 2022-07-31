use dotrix_core as dotrix;
use dotrix_math::{perspective, InnerSpace, Mat4, Point3, Quat, Rad, Rotation3, Vec3};
use std::f32::consts::PI;

/// Camera management service
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    /// Distance between the camera and a target
    pub distance: f32,
    /// Pan angle in horizontal plane (ex. y_angle)
    pub pan: f32,
    /// Tilt Angle (ex. xz_angle)
    pub tilt: f32,
    /// Roll angle
    pub roll: f32,
    /// Camera target coordinate
    pub target: Vec3,
    /// Camera position (if set, distance, pan and tilt properties will be ignored)
    pub position: Option<Vec3>,
    /// Field of View
    pub fov: f32,
    /// Near plane
    pub near_plane: f32,
    /// Far plane
    pub far_plane: f32,
}

impl Camera {
    /// Creates new Camera instance
    pub fn new() -> Self {
        let distance = 15.0;
        let pan = -PI / 2.0;
        let tilt = PI / 4.0;
        let roll = 0.0;
        let target = Vec3::new(0.0, 5.0, 0.0);
        let fov = 1.1;
        let near_plane = 0.0625;
        let far_plane = 524288.06;

        Self {
            distance,
            pan,
            tilt,
            roll,
            target,
            position: None,
            fov,
            near_plane,
            far_plane,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position.as_ref().copied().unwrap_or_else(|| {
            let dy = self.distance * self.tilt.sin();
            let dxz = self.distance * self.tilt.cos();
            let dx = dxz * self.pan.cos();
            let dz = dxz * self.pan.sin();
            Vec3::new(self.target.x + dx, self.target.y + dy, self.target.z + dz)
        })
    }

    pub fn view_matrix(&self) -> Mat4 {
        let position = self.position();
        let target = self.target;
        let direction = (target - position).normalize();
        let roll = Quat::from_axis_angle(direction, Rad(self.roll));
        let camera_right = direction.cross(Vec3::unit_y());
        let camera_up = roll * camera_right.cross(direction);

        Mat4::look_at_rh(
            Point3::new(position.x, position.y, position.z),
            Point3::new(target.x, target.y, target.z),
            camera_up,
        )
    }

    pub fn proj_matrix(&self, aspect_ratio: f32) -> Mat4 {
        perspective(Rad(self.fov), aspect_ratio, self.near_plane, self.far_plane)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Uniform {
    pub proj: [f32; 16],
    pub view: [f32; 16],
}

#[derive(Default)]
pub struct Extension {
    /// Initial camera settings
    pub camera: Camera,
    // NOTE: there will settings for camera control
}

impl dotrix::Extension for Extension {
    fn add_to(&self, manager: &mut dotrix::Manager) {
        let camera = self.camera.clone();
        manager.store(camera);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
