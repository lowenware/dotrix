use super::Color;

mod ambient_light;
pub use ambient_light::AmbientLight;

mod simple_light;
pub use simple_light::SimpleLight;
mod raw_simple_light;
use raw_simple_light::RawSimpleLight;

mod dir_light;
pub use dir_light::DirLight;
mod raw_dir_light;
use raw_dir_light::RawDirLight;

mod point_light;
pub use point_light::PointLight;
mod raw_point_light;
use raw_point_light::RawPointLight;

mod spot_light;
pub use spot_light::SpotLight;
mod raw_spot_light;
use raw_spot_light::RawSpotLight;

const MAX_LIGHTS: usize = 10;

pub trait Light<T> {
    fn to_raw(&self) -> T;
}

/// Uniform structure for lights representation in shader
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightUniform {
    pub ambient: Color,
    pub count: [u32; 4],
    pub dir_lights: [RawDirLight; MAX_LIGHTS],
    pub point_lights: [RawPointLight; MAX_LIGHTS],
    pub simple_lights: [RawSimpleLight; MAX_LIGHTS],
    pub spot_lights: [RawSpotLight; MAX_LIGHTS],
}

unsafe impl bytemuck::Zeroable for LightUniform {}
unsafe impl bytemuck::Pod for LightUniform {}

impl LightUniform {
    pub fn push_dir_light(&mut self, light: RawDirLight) { // TODO: less code duplication
        let i = self.count[0] as usize;
        if i < MAX_LIGHTS {
            self.dir_lights[i] = light;
            self.count[0] = i as u32 + 1;
        }
    }

    pub fn push_point_light(&mut self, light: RawPointLight) {
        let i = self.count[1] as usize;
        if i < MAX_LIGHTS {
            self.point_lights[i] = light;
            self.count[1] = i as u32 + 1;
        }
    }


    pub fn push_simple_light(&mut self, light: RawSimpleLight) {
        let i = self.count[2] as usize;
        if i < MAX_LIGHTS {
            self.simple_lights[i] = light;
            self.count[2] = i as u32 + 1;
        }
    }

    pub fn push_spot_light(&mut self, light: RawSpotLight) {
        let i = self.count[3] as usize;
        if i < MAX_LIGHTS {
            self.spot_lights[i] = light;
            self.count[3] = i as u32 + 1;
        }
    }
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            ambient: Color::black(),
            count: [0; 4],
            dir_lights: [RawDirLight {..Default::default()}; MAX_LIGHTS],
            point_lights: [RawPointLight {..Default::default()}; MAX_LIGHTS],
            simple_lights: [RawSimpleLight{..Default::default()}; MAX_LIGHTS],
            spot_lights: [RawSpotLight {..Default::default()}; MAX_LIGHTS],
        }
    }
}