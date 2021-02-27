use super::{ Color };
use dotrix_math::{ Vec4 };

/// Internal struct that is passed to shaders
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawPointLight {
    /// Light source position
    pub position: Vec4,
    /// Light color
    pub color: Color,
    /// Constant light attenuation
    pub a_constant: f32,
    /// Linear light attenuation
    pub a_linear: f32,
    /// Quadratic light attenuation
    pub a_quadratic: f32,
    /// Data padding (unused)
    pub padding: f32,
}

impl Default for RawPointLight {
    fn default() -> Self {
        Self {
            position: Vec4::new(0.0, 0.0, 0.0, 1.0),
            color: Color::white(),
            a_constant: 1.0,
            a_linear: 1.0,
            a_quadratic: 1.0,
            padding: 0.0,
        }
    }
}

unsafe impl bytemuck::Zeroable for RawPointLight {}
unsafe impl bytemuck::Pod for RawPointLight {}
