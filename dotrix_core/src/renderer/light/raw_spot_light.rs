use super::{ Color };
use dotrix_math::{ Vec4 };

/// Internal struct that is passed to shaders
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawSpotLight {
    /// Light source position
    pub position: Vec4,
    /// Light source direction
    pub direction: Vec4,
    /// Light source color
    pub color: Color,
    /// Light source cut off
    pub cut_off: f32,
    /// Light source outer cut off
    pub outer_cut_off: f32,
    /// structure padding
    pub padding: [f32; 2],
}

impl Default for RawSpotLight {
    fn default() -> Self {
        Self {
            position: Vec4::new(0.0, 0.0, 0.0, 1.0),
            direction: Vec4::new(0.0, 0.0, 0.0, 1.0),
            color: Color::white(),
            cut_off: 0.0,
            outer_cut_off: 0.0,
            padding: [0.0; 2],
        }
    }
}

unsafe impl bytemuck::Zeroable for RawSpotLight {}
unsafe impl bytemuck::Pod for RawSpotLight {}
