use dotrix_math::Vec4;

const MAX_LIGHTS: usize = 10;

/// Component to be added to entities
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Light {
    pub position: Vec4,
    pub color: Vec4,
}

impl Light {
    pub fn white(position: [f32; 3]) -> Self {
        Self {
            position: Vec4::new(position[0], position[1], position[2], 1.0),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

unsafe impl bytemuck::Zeroable for Light {}
unsafe impl bytemuck::Pod for Light {}

/// Uniform structure for lights representation in shader
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightUniform {
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
            length: [0; 4],
            light_source: [Light::white([0.0, 0.0, 0.0]); MAX_LIGHTS],
        }
    }
}
