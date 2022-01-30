//! Mouse ray implementation
use crate::ecs::{Const, Mut};
use crate::{Camera, Input, Window};
use dotrix_math::{InnerSpace, SquareMatrix, Vec2, Vec3, Vec4};

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
    /// Returns optional ray length in near and far intersection points
    pub fn intersect_aligned_box(&self, bounds: [Vec3; 2]) -> Option<(f32, f32)> {
        if let Some(origin) = self.origin.as_ref() {
            if let Some(inverted) = self.inverted.as_ref() {
                let x_min = (bounds[self.sign[0] as usize].x - origin.x) * inverted.x;
                let x_max = (bounds[1 - self.sign[0] as usize].x - origin.x) * inverted.x;

                let y_min = (bounds[self.sign[1] as usize].y - origin.y) * inverted.y;
                let y_max = (bounds[1 - self.sign[1] as usize].y - origin.y) * inverted.y;

                if x_min > y_max || y_min > x_max {
                    return None;
                }

                let t_min = if y_min > x_min { y_min } else { x_min };
                let t_max = if y_max < x_max { y_max } else { x_max };

                let z_min = (bounds[self.sign[2] as usize].z - origin.z) * inverted.z;
                let z_max = (bounds[1 - self.sign[2] as usize].z - origin.z) * inverted.z;

                if t_min > z_max || z_min > t_max {
                    return None;
                }

                let t_min = if z_min > t_min { z_min } else { t_min };
                let t_max = if z_max < t_max { z_max } else { t_max };

                return Some((t_min, t_max));
            }
        }
        None
    }
}

/// Mouse ray calculation system
pub fn calculate(
    mut ray: Mut<Ray>,
    camera: Const<Camera>,
    input: Const<Input>,
    window: Const<Window>,
) {
    ray.direction = input.mouse_position().map(|mouse| {
        let viewport = window.inner_size();
        let ray = Ray::normalized_device_coords(mouse, viewport.x as f32, viewport.y as f32);

        // eye coordinates
        let mut ray = camera.proj().invert().unwrap() * ray;

        ray.z = -1.0;
        ray.w = 0.0;
        // world coordinates
        let ray = camera.view().invert().unwrap() * ray;
        Vec3::new(ray.x, ray.y, ray.z).normalize()
    });

    let inverted = ray.direction.as_ref().map(|d| 1.0_f32 / d);

    ray.origin = Some(camera.position());

    if let Some(inverted) = inverted.as_ref() {
        ray.sign[0] = if inverted.x < 0.0 { 1 } else { 0 };
        ray.sign[1] = if inverted.y < 0.0 { 1 } else { 0 };
        ray.sign[2] = if inverted.z < 0.0 { 1 } else { 0 };
    }

    ray.inverted = inverted;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_intersection() {
        let direction = Vec3::new(0.0, 0.0, -1.0);
        let inverted = 1.0_f32 / direction;
        let sign = [
            if inverted.x < 0.0 { 1 } else { 0 },
            if inverted.y < 0.0 { 1 } else { 0 },
            if inverted.z < 0.0 { 1 } else { 0 },
        ];
        let ray = Ray {
            direction: Some(direction),
            origin: Some(Vec3::new(0.0, 0.0, 10.0)),
            inverted: Some(inverted),
            sign,
        };

        let res =
            ray.intersect_aligned_box([Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)]);

        assert!(res.is_some());
        let (t_min, t_max) = res.unwrap();

        assert_eq!(t_min.round() as i32, 9);
        assert_eq!(t_max.round() as i32, 11);
    }
}
