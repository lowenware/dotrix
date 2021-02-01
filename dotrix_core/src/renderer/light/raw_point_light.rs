use super::{ Color };
use dotrix_math::{ Vec4 };

/// Internal struct that is passed to shaders
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawPointLight {
    pub position: Vec4,
    pub color: Color, // Vec4

    // attenuation
    pub a_constant: f32,
    pub a_linear: f32,
    pub a_quadratic: f32,
    pub unused: f32,
}

impl Default for RawPointLight {
    fn default() -> Self {
        Self {
            position: Vec4::new(0.0, 0.0, 0.0, 1.0),
            color: Color::white(),
            a_constant: 1.0,
            a_linear: 1.0,
            a_quadratic: 1.0,
            unused: 1.0,
        }
    }
}

unsafe impl bytemuck::Zeroable for RawPointLight {}
unsafe impl bytemuck::Pod for RawPointLight {}