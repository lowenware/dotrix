use dotrix::{
    ecs::{ Const, Mut },
    services::{ Camera, Input, Renderer },
    math::{ InnerSpace, SquareMatrix, Vec2, Vec3, Vec4 },
};

#[derive(Default)]
pub struct MouseRay {
    pub ray: Option<Vec3>,
}

impl MouseRay {
    pub fn normalized_device_coords(
        mouse: &Vec2,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Vec4 {

        let x = (2.0 * mouse.x) / viewport_width - 1.0;
        let y = 1.0 - (2.0 * mouse.y) / viewport_height;
        Vec4::new(x, y, -1.0, 1.0)
    }
}

pub fn mouse_ray(
    mut mouse_ray: Mut<MouseRay>,
    camera: Const<Camera>,
    input: Const<Input>,
    renderer: Const<Renderer>,
) {
    mouse_ray.ray = input.mouse_position().map(|mouse| {
        let (viewport_width, viewport_height) = renderer.display_size();
        let ray = MouseRay::normalized_device_coords(
            mouse, viewport_width as f32, viewport_height as f32);
        // eye coordinates
        let mut ray = renderer.projection.invert().unwrap() * ray;
        ray.z = -1.0;
        ray.w = 1.0;
        // world coordinates
        let ray = camera.view().invert().unwrap() * ray;
        Vec3::new(-ray.x, -ray.y, -ray.z).normalize()
    });
}
