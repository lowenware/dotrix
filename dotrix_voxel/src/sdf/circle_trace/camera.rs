use dotrix_core::{
    ecs::{Const, Mut, System},
    renderer::Buffer,
    Application, Camera, Globals, Renderer, Window,
};
use dotrix_math::SquareMatrix;

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub(super) struct Uniform {
    proj_view: [[f32; 4]; 4],
    static_camera_trans: [[f32; 4]; 4],
    pos: [f32; 4],
    screen_resolution: [f32; 2],
    fov: f32,
    padding: [f32; 1],
}

unsafe impl bytemuck::Zeroable for Uniform {}
unsafe impl bytemuck::Pod for Uniform {}

pub struct CameraBuffer {
    pub uniform: Buffer,
}

impl Default for CameraBuffer {
    fn default() -> Self {
        Self {
            uniform: Buffer::uniform("SDF Camera Buffer"),
        }
    }
}

/// startup system
pub(super) fn startup(mut globals: Mut<Globals>) {
    globals.set(CameraBuffer::default());
}

/// startup system
pub(super) fn load(
    renderer: Const<Renderer>,
    camera: Const<Camera>,
    window: Const<Window>,
    mut globals: Mut<Globals>,
) {
    let proj_mx = *camera.proj();
    let view_mx = camera.view_matrix();
    let static_camera_mx = camera.view_matrix_static().invert().unwrap();
    let camera_pos = camera.position();
    let inner_size = window.inner_size();
    let uniform = Uniform {
        proj_view: (proj_mx * view_mx).into(),
        static_camera_trans: static_camera_mx.into(),
        pos: [camera_pos[0], camera_pos[1], camera_pos[2], 1.],
        screen_resolution: [inner_size[0] as f32, inner_size[1] as f32],
        fov: camera.fov,
        padding: Default::default(),
    };
    println!("screen_resolution: {:?}", uniform.screen_resolution);
    if let Some(uniform_buffer) = globals.get_mut::<CameraBuffer>() {
        renderer.load_buffer(
            &mut uniform_buffer.uniform,
            bytemuck::cast_slice(&[uniform]),
        );
    }
}

pub(super) fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(load));
}
