use super::{ Color };
use dotrix_math::{ Vec4 };

/// Internal struct that is passed to shaders
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawDirLight {
    /// Light direction
    pub direction: Vec4,
    /// Light color
    pub color: Color
}

impl Default for RawDirLight {
    fn default() -> Self {
        Self {
            direction: Vec4::new(0.0, 0.0, 0.0, 1.0),
            color: Color::white(),
        }
    }
}

unsafe impl bytemuck::Zeroable for RawDirLight {}
unsafe impl bytemuck::Pod for RawDirLight {}
