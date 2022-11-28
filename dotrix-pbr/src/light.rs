use std::task::Context;

use dotrix_core as dotrix;
use dotrix_ecs::World;
use dotrix_gpu::{Buffer, Gpu, Texture, TextureView};
use dotrix_math::Vec3;
use dotrix_types::{Color, Id};

#[derive(Debug)]
pub enum Mode {
    Ambient,
    Directional {
        direction: Vec3,
    },
    Simple {
        position: Vec3,
    },
    Point {
        position: Vec3,
        constant: f32,
        linear: f32,
        quadratic: f32,
    },
    Spot {
        position: Vec3,
        direction: Vec3,
        cut_off: f32,
        outer_cut_off: f32,
    },
}

#[derive(Debug)]
pub struct Light {
    /// The color of light, alpha channel is being used as intensity
    pub color: Color,
    /// Light Mode
    pub mode: Mode,
    /// Light On/Off toggle
    pub enabled: bool,
    /// Shadow On/Off toggle
    pub shadow: bool,
}

impl Light {
    pub fn ambient() -> Self {
        Self {
            mode: Mode::Ambient,
            ..Default::default()
        }
    }

    pub fn directional(direction: Vec3) -> Self {
        Self {
            mode: Mode::Directional { direction },
            ..Default::default()
        }
    }

    pub fn simple(position: Vec3) -> Self {
        Self {
            mode: Mode::Simple { position },
            ..Default::default()
        }
    }

    pub fn point(position: Vec3, constant: f32, linear: f32, quadratic: f32) -> Self {
        Self {
            mode: Mode::Point {
                position,
                constant,
                linear,
                quadratic,
            },
            ..Default::default()
        }
    }

    pub fn spot(position: Vec3, direction: Vec3, cut_off: f32, outer_cut_off: f32) -> Self {
        Self {
            mode: Mode::Spot {
                position,
                direction,
                cut_off,
                outer_cut_off,
            },
            ..Default::default()
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color.r = color.r;
        self.color.g = color.g;
        self.color.b = color.b;
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.color.a = intensity;
        self
    }

    pub fn shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            color: Color::rgba(1.0, 0.98, 0.85, 0.5),
            enabled: true,
            mode: Mode::Ambient,
            shadow: true,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Uniform {
    /// 0-2: rgb, 3: intensity
    pub color: [f32; 4],
    /// 0: enabled, 1: mode, 2: shadow, 3: reserved
    pub options: [u32; 4],
    /// Mode::Ambient -> _
    /// Mode::Directional -> 0-2: direction
    /// Mode::Simple -> 0-2: position
    /// Mode::Point -> 0-2: position
    /// Mode::Spot -> 0-2: position, 3: cut_off
    pub mode_options_1: [f32; 4],
    /// Mode::Ambient -> _
    /// Mode::Directional -> _
    /// Mode::Simple -> _
    /// Mode::Point -> 0: constant, 1: linear, 2: quadratic
    /// Mode::Spot -> 0-2: direction, 3: outer_cut_off
    pub mode_options_2: [f32; 4],
}

unsafe impl bytemuck::Pod for Uniform {}
unsafe impl bytemuck::Zeroable for Uniform {}

impl From<&Light> for Uniform {
    fn from(light: &Light) -> Self {
        let color = light.color.into();
        let (mode, mode_options_1, mode_options_2) = match light.mode {
            Mode::Ambient => (0, [0.0; 4], [0.0; 4]),
            Mode::Directional { direction } => {
                (1, [direction.x, direction.y, direction.z, 0.0], [0.0; 4])
            }
            Mode::Simple { position } => (2, [position.x, position.y, position.z, 0.0], [0.0; 4]),
            Mode::Point {
                position,
                constant,
                linear,
                quadratic,
            } => (
                3,
                [position.x, position.y, position.z, 0.0],
                [constant, linear, quadratic, 0.0],
            ),
            Mode::Spot {
                position,
                direction,
                cut_off,
                outer_cut_off,
            } => (
                4,
                [position.x, position.y, position.z, cut_off],
                [direction.x, direction.y, direction.z, outer_cut_off],
            ),
        };
        let options = [
            if light.enabled { 1 } else { 0 },
            mode,
            if light.shadow { 1 } else { 0 },
            0,
        ];
        Self {
            color,
            options,
            mode_options_1,
            mode_options_2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Data {
    pub number_of_lights: u32,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct LoadTask {}

impl LoadTask {
    pub fn new() -> Self {
        Self::default()
    }
}

impl dotrix::Task for LoadTask {
    type Context = (
        dotrix::Any<crate::Buffers>,
        dotrix::Ref<Gpu>,
        dotrix::Ref<World>,
    );

    type Output = Data;

    fn run(&mut self, (buffers, gpu, world): Self::Context) -> Self::Output {
        let mut number_of_lights = 0;
        let light_buffer = gpu.extract(&buffers.light);

        for (light,) in world.query::<(&Light,)>() {
            if light.enabled {
                let uniform = Uniform::from(light);
                gpu.write_buffer(
                    light_buffer,
                    (std::mem::size_of::<Uniform>() as u32 * number_of_lights) as u64,
                    bytemuck::cast_slice(&[uniform]),
                );
                number_of_lights += 1;
            }
        }

        Data { number_of_lights }
    }
}
