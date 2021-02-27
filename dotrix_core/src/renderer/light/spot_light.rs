use super::{ Color, Light, RawSpotLight };
use dotrix_math::{ Vec3, Vec4 };

#[derive(Clone, Debug)]
/// Component to be added to entities
pub struct SpotLight {
    /// Is light source enabled
    pub enabled: bool,
    /// Light source position
    pub position: Vec3,
    /// Light source direction
    pub direction: Vec3,
    /// Light source color
    pub color: Color,
    /// Light source intensity
    pub intensity: f32,
    /// Light source cut off
    pub cut_off: f32,
    /// Light source outer cut off
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
            padding: [0.0; 2],
        }
    }
}
