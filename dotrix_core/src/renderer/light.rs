use super::Color;
use dotrix_math::Vec3;

const MAX_LIGHTS: usize = 10;

pub struct AmbientLight {
    pub color: Color,
}

impl AmbientLight {
    pub fn new(color: Color) -> Self {
        Self {
            color
        }
    }
}

/// Component to be added to entities
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Light {
    pub position: Vec3,
    pub intensity: f32, // Do not sort, must be exactly like that
    pub color: Color,
}

impl Light {
    pub fn white(position: Vec3) -> Self {
        Self {
            position,
            intensity: 1.0,
            color: Color::white(),
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            intensity: 1.0,
            color: Color::white(),
        }
    }
}

unsafe impl bytemuck::Zeroable for Light {}
unsafe impl bytemuck::Pod for Light {}

/// Uniform structure for lights representation in shader
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightUniform {
    pub ambient: Color,
    pub length: [u32; 4],
    pub light_source: [Light; MAX_LIGHTS],
}

unsafe impl bytemuck::Zeroable for LightUniform {}
unsafe impl bytemuck::Pod for LightUniform {}

impl LightUniform {
    pub fn push(&mut self, light: Light) {
        let i = self.length[0] as usize;
        if i < MAX_LIGHTS {
            self.light_source[i] = light;
            self.length[0] = i as u32 + 1;
        }
    }
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            ambient: Color::black(),
            length: [0; 4],
            light_source: [Light::white(Vec3::new(0.0, 0.0, 0.0)); MAX_LIGHTS],
        }
    }
}
