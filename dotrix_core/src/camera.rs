//! Dotrix camera implementation
use crate::{
    ecs::{ Mut, Const },
    services::{ Frame, Globals, Input, Renderer, Window },
    renderer::{ UniformBuffer },
};

use dotrix_math::{ Mat4, Point3, Vec3, perspective, Rad };
use std::f32::consts::PI;

const ROTATE_SPEED: f32 = PI / 10.0;
const ZOOM_SPEED: f32 = 10.0;

/// Projection View matrix
#[derive(Default)]
pub struct ProjView {
    /// Uniform Buffer of ProjView matrix
    pub uniform: UniformBuffer,
}

/// Camera management service
pub struct Camera {
    /// Distance between the camera and a target
    pub distance: f32,
    /// Angle around the Y axis
    pub y_angle: f32,
    /// Angle in horizontal plane
    pub xz_angle: f32,
    /// Camera target coordinate
    pub target: Point3,
    /// View matri
    pub view: Option<Mat4>,
    /// Field of View
    pub fov: f32,
    /// Near plane
    pub near_plane: f32,
    /// Far plane
    pub far_plane: f32,
    /// Projection matrix
    pub proj: Option<Mat4>,
}

impl Camera {
    /// Creates new Camera instance
    pub fn new() -> Self {
        let distance = 15.0;
        let y_angle = -PI / 2.0;
        let xz_angle = PI / 4.0;
        let target = Point3::new(0.0, 5.0, 0.0);
        let fov = 1.1;
        let near_plane = 0.0625;
        let far_plane = 524288.06;

        Self {
            distance,
            y_angle,
            xz_angle,
            target,
            view: None,
            fov,
            near_plane,
            far_plane,
            proj: None
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
        let dy = self.distance * self.xz_angle.sin();
        let dxz = self.distance * self.xz_angle.cos();
        let dx = dxz * self.y_angle.cos();
        let dz = dxz * self.y_angle.sin();
        Vec3::new(self.target.x + dx, self.target.y + dy, self.target.z + dz)
    }

    /// Returns view matrix
    pub fn view_matrix(&self) -> Mat4 {
        let dy = self.distance * self.xz_angle.sin();
        let dxz = self.distance * self.xz_angle.cos();
        let dx = dxz * self.y_angle.cos();
        let dz = dxz * self.y_angle.sin();
        let position = Point3::new(self.target.x + dx, self.target.y + dy, self.target.z + dz);

        let (position, target) = if self.distance > 0.0 {
            (position, self.target)
        } else {
            (self.target, position)
        };

        Mat4::look_at(position, target, Vec3::unit_y())
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
    let proj_view = ProjView::default();
    globals.set(proj_view);
}

/// Loads the ProjView binding with current matrix value
pub fn bind(
    mut globals: Mut<Globals>,
    mut camera: Mut<Camera>,
    renderer: Const<Renderer>,
    window: Const<Window>
) {
    // Calculate projection matrix
    if camera.proj.is_none() {
        camera.proj = Some(perspective(
            Rad(camera.fov),
            window.aspect_ratio(),
            camera.near_plane,
            camera.far_plane
        ));
    }

    // Calculate view matrix
    camera.view = Some(camera.view_matrix());

    // Set uniform buffer with proj x view matrix
    if let Some(proj_view) = globals.get_mut::<ProjView>() {
        let matrix = camera.proj.as_ref().unwrap() * camera.view.as_ref().unwrap();
        let matrix_raw = AsRef::<[f32; 16]>::as_ref(&matrix);

        renderer.load_uniform_buffer(&mut proj_view.uniform, bytemuck::cast_slice(matrix_raw));
    }
}

/// Updates the Project matrix with new aspect ratio
pub fn resize(mut camera: Mut<Camera>, window: Const<Window>) {
    // Calculate new projection matrix
    camera.proj = Some(perspective(
        Rad(camera.fov),
        window.aspect_ratio(),
        camera.near_plane,
        camera.far_plane
    ));
}

/// System controlling camera with mouse
pub fn control(mut camera: Mut<Camera>, input: Const<Input>, frame: Const<Frame>) {
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
}


