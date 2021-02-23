use super::{ Color, Light, RawSpotLight };
use dotrix_math::{ Vec2, Vec3, Vec4 };

#[derive(Clone, Debug)]
/// Component to be added to entities
pub struct SpotLight {
    pub enabled: bool,
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub cut_off: f32,
    pub outer_cut_off: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            enabled: true,
            position: Vec3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
            cut_off: 0.8,
            outer_cut_off: 0.65,
        }
    }
}

impl Light<RawSpotLight> for SpotLight {
    fn to_raw(&self) -> RawSpotLight {
        RawSpotLight {
            position: Vec4 {
                x: self.position.x,
                y: self.position.y,
                z: self.position.z,
                w: 1.0,
            },
            direction: Vec4 {
                x: self.direction.x,
                y: self.direction.y,
                z: self.direction.z,
                w: 1.0,
            },
            color: self.color.mul_f32(self.intensity),
            cut_off: self.cut_off,
            outer_cut_off: self.outer_cut_off,
            unused: Vec2::new(0.0, 0.0),
        }
    }
}
