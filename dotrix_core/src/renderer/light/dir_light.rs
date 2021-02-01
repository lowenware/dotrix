use super::{ Color, Light, RawDirLight };
use dotrix_math::{ Vec3, Vec4 };

#[derive(Clone, Debug)]
/// Component to be added to entities
pub struct DirLight {
    pub enabled: bool,
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Default for DirLight {
    fn default() -> Self {
        Self {
            enabled: true,
            direction: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
        }
    }
}

impl Light<RawDirLight> for DirLight {
    fn to_raw(&self) -> RawDirLight {
        RawDirLight {
            direction: Vec4 {
                x: self.direction.x,
                y: self.direction.y,
                z: self.direction.z,
                w: 1.0,
            },
            color: self.color.mul_f32(self.intensity),
        }
    }
}
