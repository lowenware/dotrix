use super::{ Color };
use dotrix_math::{ Vec4 };

/// Internal struct that is passed to shaders
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawSimpleLight {
    pub position: Vec4,
    pub color: Color, // Vec4
}

impl Default for RawSimpleLight {
    fn default() -> Self {
        Self {
            position: Vec4::new(0.0, 0.0, 0.0, 1.0),
            color: Color::white(),
        }
    }
}

unsafe impl bytemuck::Zeroable for RawSimpleLight {}
unsafe impl bytemuck::Pod for RawSimpleLight {}