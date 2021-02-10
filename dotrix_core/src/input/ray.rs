use crate::{
    ecs::{ Const, Mut },
    services::{ Camera, Input, Renderer },
};
use dotrix_math::{ InnerSpace, SquareMatrix, Vec2, Vec3, Vec4 };

/// Represents ray and provides method for various calculations
#[derive(Default)]
pub struct Ray {
    /// Ray direction
    pub direction: Option<Vec3>,
    /// Inverted ray direction
    pub inverted: Option<Vec3>,
    /// Ray origin point
    pub origin: Option<Vec3>,
    /// Index array according to ray axis signs
    pub sign: [u8; 3],
}

impl Ray {
    /// Calculates normalized device coords. Pointer argument can be mouse coordinates or, for
    /// example middle moint of the screen (crosshair)
    pub fn normalized_device_coords(
        pointer: &Vec2,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Vec4 {

        let x = (2.0 * pointer.x) / viewport_width - 1.0;
        let y = 1.0 - (2.0 * pointer.y) / viewport_height;
        Vec4::new(x, y, -1.0, 1.0)
    }

    /// Calculate intersection with an axis aligned box
    /// Returns optional positions of far and near intersection points
    pub fn intersect_box(
        &self,
        bounds: [Vec3; 2],
    ) -> Option<(Vec3, Vec3)> {

        if let Some(origin) = self.origin.as_ref() {
            if let Some(inverted) = self.inverted.as_ref() {

                let x_min = (bounds[self.sign[0] as usize].x - origin.x) * inverted.x;
                let x_max = (bounds[1 - self.sign[0] as usize].x - origin.x) * inverted.x;

                let y_min = (bounds[self.sign[1] as usize].y - origin.y) * inverted.y;
                let y_max = (bounds[1 - self.sign[1] as usize].y - origin.y) * inverted.y;

                if x_min > y_max || y_min > x_max {
                    return None;
                }

                let min = if y_min > x_min { y_min } else { x_min };
                let max = if y_max < x_max { y_max } else { x_max };

                let z_min = (bounds[self.sign[2] as usize].z - origin.z) * inverted.z; 
                let z_max = (bounds[1 - self.sign[2] as usize].z - origin.z) * inverted.z; 

                if min > z_max || z_min > max {
                    return None; 
                }

                let min = if z_min > min { z_min } else { min };
                let max = if z_max < max { z_max } else { max };
 
                return Some((Vec3::new(min, y_min, z_min), Vec3::new(max, y_max, z_max)));
            }
        }
        None
    }
}

/// Mouse ray calculation system
pub fn mouse_ray(
    mut ray: Mut<Ray>,
    camera: Const<Camera>,
    input: Const<Input>,
    renderer: Const<Renderer>,
) {
    ray.direction = input.mouse_position().map(|mouse| {
        let (viewport_width, viewport_height) = renderer.display_size();
        let ray = Ray::normalized_device_coords(
            mouse, viewport_width as f32, viewport_height as f32);

        // eye coordinates
        let mut ray = renderer.projection.invert().unwrap() * ray;
        ray.z = -1.0;
        ray.w = 0.0;
        // world coordinates
        let ray = camera.view().invert().unwrap() * ray;
        Vec3::new(ray.x, ray.y, ray.z).normalize()
    });

    println!("dir {:?}", ray.direction);

    let inverted = ray.direction.as_ref().map(|d| 1.0_f32 / d);

    ray.origin = Some(camera.position());

    if let Some(inverted) = inverted.as_ref() {
        ray.sign[0] = if inverted.x < 0.0 { 1 } else { 0 };
        ray.sign[1] = if inverted.y < 0.0 { 1 } else { 0 };
        ray.sign[2] = if inverted.z < 0.0 { 1 } else { 0 };
    }

    ray.inverted = inverted;
}
