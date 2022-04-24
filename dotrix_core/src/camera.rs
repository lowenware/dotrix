//! Dotrix camera implementation
use crate::ecs::{Const, Mut};
use crate::renderer::Buffer;
use crate::{Frame, Globals, Input, Renderer, Window};

use dotrix_math::{perspective, InnerSpace, Mat4, Point3, Quat, Rad, Rotation3, Vec3};
use std::f32::consts::PI;

const ROTATE_SPEED: f32 = PI / 10.0;
const ZOOM_SPEED: f32 = 10.0;

/// Projection View matrix
#[derive(Default)]
pub struct ProjView {
    /// Uniform Buffer of ProjView matrix
    pub uniform: Buffer,
}

/// Camera management service
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
    /// Projection matrix
    pub proj: Option<Mat4>,
    /// View matri
    pub view: Option<Mat4>,
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
            view: None,
            fov,
            near_plane,
            far_plane,
            proj: None,
        }
    }

    /// Returns view matrix with zero transition
    ///
    /// It is useful for sky boxes and domes
    pub fn view_matrix_static(&self) -> Mat4 {
        let mut view_static = self.view.expect("View matrix must be set");
        view_static.w.x = 0.0;
        view_static.w.y = 0.0;
        view_static.w.z = 0.0;
        view_static
    }

    /// Returns calculated camera position
    pub fn position(&self) -> Vec3 {
        self.position.as_ref().copied().unwrap_or_else(|| {
            let dy = self.distance * self.tilt.sin();
            let dxz = self.distance * self.tilt.cos();
            let dx = dxz * self.pan.cos();
            let dz = dxz * self.pan.sin();
            Vec3::new(self.target.x + dx, self.target.y + dy, self.target.z + dz)
        })
    }

    /// Returns normalized direction vector
    pub fn direction(&self) -> Vec3 {
        let position = self.position();
        (Vec3::new(self.target.x, self.target.y, self.target.z) - position).normalize()
    }

    /// Returns view matrix
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

    /// Returns a reference to view matrix
    pub fn view(&self) -> &Mat4 {
        self.view.as_ref().expect("View matrix must be set")
    }

    /// Returns a reference to projection matrix
    pub fn proj(&self) -> &Mat4 {
        self.proj.as_ref().expect("Projection matrix must be set")
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// Camera startup system
/// Initializes the ProjView binding
pub fn startup(mut globals: Mut<Globals>) {
    let proj_view = ProjView {
        uniform: Buffer::uniform("ProjView buffer"),
    };
    globals.set(proj_view);
}

/// Loads the ProjView binding with current matrix value
pub fn load(
    mut globals: Mut<Globals>,
    mut camera: Mut<Camera>,
    renderer: Const<Renderer>,
    window: Const<Window>,
) {
    // Calculate projection matrix
    if camera.proj.is_none() {
        camera.proj = Some(perspective(
            Rad(camera.fov),
            window.aspect_ratio(),
            camera.near_plane,
            camera.far_plane,
        ));
    }

    // Calculate view matrix
    camera.view = Some(camera.view_matrix());

    // Set uniform buffer with proj x view matrix
    if let Some(proj_view) = globals.get_mut::<ProjView>() {
        let matrix = camera.proj.as_ref().unwrap() * camera.view.as_ref().unwrap();
        let matrix_raw = AsRef::<[f32; 16]>::as_ref(&matrix);

        renderer.load_buffer(&mut proj_view.uniform, bytemuck::cast_slice(matrix_raw));
    }
}

/// Updates the Project matrix with new aspect ratio
pub fn resize(mut camera: Mut<Camera>, window: Const<Window>) {
    // Calculate new projection matrix
    camera.proj = Some(perspective(
        Rad(camera.fov),
        window.aspect_ratio(),
        camera.near_plane,
        camera.far_plane,
    ));
}

/// System controlling camera with mouse
pub fn control(mut camera: Mut<Camera>, input: Const<Input>, frame: Const<Frame>) {
    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    let distance = camera.distance - ZOOM_SPEED * mouse_scroll * time_delta;
    camera.distance = if distance > -1.0 { distance } else { -1.0 };

    camera.pan += mouse_delta.x * ROTATE_SPEED * time_delta;

    let tilt = camera.tilt + mouse_delta.y * ROTATE_SPEED * time_delta;
    let half_pi = PI / 2.0;

    camera.tilt = if tilt >= half_pi {
        half_pi - 0.01
    } else if tilt <= -half_pi {
        -half_pi + 0.01
    } else {
        tilt
    };
}
