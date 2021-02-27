use super::{ Color, Light, RawPointLight };
use dotrix_math::{ Vec3, Vec4 };

/// Component to be added to entities
#[derive(Clone, Debug)]
pub struct PointLight {
    /// Is light source enabled
    pub enabled: bool,
    /// Light source position
    pub position: Vec3,
    /// Light color
    pub color: Color,
    /// Light source intencity
    pub intensity: f32,
    /// Constant light
    pub constant: f32,
    /// Linear light
    pub linear: f32,
    /// Quadratic light
    pub quadratic: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            enabled: true,
            position: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
            constant: 1.0,
            linear: 0.35,
            quadratic: 0.44,
        }
    }
}

impl Light<RawPointLight> for PointLight {
    fn to_raw(&self) -> RawPointLight {
        RawPointLight {
            position: Vec4 {
                x: self.position.x,
                y: self.position.y,
                z: self.position.z,
                w: 1.0,
            },
            color: self.color.mul_f32(self.intensity),

            a_constant: self.constant,
            a_linear: self.linear,
            a_quadratic: self.quadratic,
            padding: 1.0,
        }
    }
}
