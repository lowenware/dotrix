use super::{ Color, Light, RawSimpleLight };
use dotrix_math::{ Vec3, Vec4 };

#[derive(Clone, Debug)]
/// Component to be added to entities
pub struct SimpleLight {
    /// Is light source enabled
    pub enabled: bool,
    /// Light source position
    pub position: Vec3,
    /// Light color
    pub color: Color,
    /// Light intensity
    pub intensity: f32,
}

impl Default for SimpleLight {
    fn default() -> Self {
        Self {
            enabled: true,
            position: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
        }
    }
}

impl Light<RawSimpleLight> for SimpleLight {
    fn to_raw(&self) -> RawSimpleLight {
        RawSimpleLight {
            position: Vec4 {
                x: self.position.x,
                y: self.position.y,
                z: self.position.z,
                w: 1.0,
            },
            color: self.color.mul_f32(self.intensity),
        }
    }
}
