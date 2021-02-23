use super::{ Color };
use dotrix_math::{ Vec2, Vec4 };

/// Internal struct that is passed to shaders
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawSpotLight {
    pub position: Vec4,
    pub direction: Vec4,
    pub color: Color, // Vec4
    pub cut_off: f32,
    pub outer_cut_off: f32,
    pub unused: Vec2,
}

impl Default for RawSpotLight {
    fn default() -> Self {
        Self {
            position: Vec4::new(0.0, 0.0, 0.0, 1.0),
            direction: Vec4::new(0.0, 0.0, 0.0, 1.0),
            color: Color::white(),
            cut_off: 0.0,
            outer_cut_off: 0.0,
            unused: Vec2::new(0.0, 0.0),
        }
    }
}

unsafe impl bytemuck::Zeroable for RawSpotLight {}
unsafe impl bytemuck::Pod for RawSpotLight {}