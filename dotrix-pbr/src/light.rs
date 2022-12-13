use std::{ops::Range, task::Context};

use dotrix_core as dotrix;
use dotrix_ecs::World;
use dotrix_gpu::{Buffer, Gpu, Texture, TextureView};
use dotrix_math::{Deg, InnerSpace, Rad, Vec3};
use dotrix_types::{Camera, Color, Id};

#[derive(Debug, Clone)]
pub enum Position {
    /// Directional ambient light
    Ambient { dir_x: f32, dir_y: f32, dir_z: f32 },
    /// Light with a source at some defined point
    Point { x: f32, y: f32, z: f32 },
}

#[derive(Debug, Clone)]
pub struct Light {
    /// The color of light, alpha channel is being used as intensity
    pub color: Color<f32>,
    /// Light Position
    pub position: Position,
    /// Light Stream Direction vector
    pub stream: Vec3,
    /// Field of View
    pub fov: Rad<f32>,
    /// Depth of light
    pub depth: Range<f32>,
    /// Light source constant attenuation
    pub blur_constant: f32,
    /// Light source linear attenuation
    pub blur_linear: f32,
    /// Light source quadratic attenuation
    pub blur_quadratic: f32,
    /// Light ray cut off
    pub cut_off_inner: f32,
    /// Light ray outer cut off
    pub cut_off_outer: f32,
    /// Light On/Off toggle
    pub enabled: bool,
    /// Shadow On/Off toggle
    pub shadow: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            color: Color::rgba(1.0, 0.98, 0.85, 0.5),
            position: Position::Ambient {
                dir_x: -3.0,
                dir_y: 1.0,
                dir_z: -2.0,
            },
            stream: Vec3::new(0.0, 0.0, 0.0),
            fov: Deg(60.0).into(),
            depth: 0.1..50.0,
            blur_constant: 0.0,
            blur_linear: 0.0,
            blur_quadratic: 0.0,
            cut_off_inner: 0.0,
            cut_off_outer: 0.0,
            enabled: true,
            shadow: true,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Uniform {
    /// ProjView matrix
    pub proj_view: [[f32; 4]; 4],
    /// rgb, a:unused
    pub color: [f32; 4],
    /// xyz, w: if w < 1.0 { dir } else { pos }
    pub pos_dir: [f32; 4],
    /// xyz, w:unused
    pub stream: [f32; 4],
    /// x:constant, y:linear, z:quadratic, w:unused
    pub blur: [f32; 4],
    /// x:cut_off, y:outer_cut_off, zw: unused
    pub cut_off: [f32; 4],
    /// x:shadow, yzw: unused
    pub options: [u32; 4],
}

unsafe impl bytemuck::Pod for Uniform {}
unsafe impl bytemuck::Zeroable for Uniform {}

impl Light {
    pub fn ambient(dir_x: f32, dir_y: f32, dir_z: f32) -> Self {
        Self {
            position: Position::Ambient {
                dir_x,
                dir_y,
                dir_z,
            },
            ..Default::default()
        }
    }

    pub fn point(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Position::Point { x, y, z },
            ..Default::default()
        }
    }

    pub fn color(mut self, color: Color<f32>) -> Self {
        self.color.r = color.r;
        self.color.g = color.g;
        self.color.b = color.b;
        self
    }

    pub fn intensity(mut self, intensity: f32) -> Self {
        self.color.a = intensity;
        self
    }

    pub fn shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }

    pub fn to_uniform(
        &self,
        scene: Vec3,
        shadow_texture_width: u32,
        shadow_texture_height: u32,
        ambient_light_distance: f32,
    ) -> Uniform {
        const RESERVED_F32: f32 = 0.0;
        const RESERVED_U32: u32 = 0;
        let intensity = self.color.a;
        let color = [
            self.color.r * intensity,
            self.color.g * intensity,
            self.color.b * intensity,
            RESERVED_F32,
        ];
        let (pos_dir, view_pos, depth_delta) = match self.position {
            Position::Ambient {
                dir_x,
                dir_y,
                dir_z,
            } => {
                let dir = Vec3::new(dir_x, dir_y, dir_z);
                let view_pos = -dir * ambient_light_distance + scene;
                (
                    [dir.x, dir.y, dir.z, 0.0],
                    [view_pos.x, view_pos.y, view_pos.z],
                    ambient_light_distance,
                )
            }
            Position::Point { x, y, z } => ([x, y, z, 2.0], [x, y, z], 0.0),
        };
        let stream = [self.stream.x, self.stream.y, self.stream.z, RESERVED_F32];
        let blur = [
            self.blur_constant,
            self.blur_linear,
            self.blur_quadratic,
            RESERVED_F32,
        ];
        let cut_off = [
            self.cut_off_inner,
            self.cut_off_outer,
            RESERVED_F32,
            RESERVED_F32,
        ];
        let options = [
            if self.shadow { 1 } else { 0 },
            RESERVED_U32,
            RESERVED_U32,
            RESERVED_U32,
        ];
        let depth = (self.depth.start + depth_delta)..(self.depth.end + depth_delta);
        let proj = Camera::lens(self.fov, depth).proj(shadow_texture_width, shadow_texture_height);
        let view = Camera::at(view_pos[0], view_pos[1], view_pos[2]).target(scene);

        Uniform {
            color,
            pos_dir,
            stream,
            blur,
            cut_off,
            options,
            proj_view: (proj * view).into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    pub number_of_lights: u32,
    pub shadows: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct LoadTask {
    shadow_texture_width: u32,
    shadow_texture_height: u32,
    ambient_light_distance: f32,
}

impl LoadTask {
    pub fn new(
        shadow_texture_width: u32,
        shadow_texture_height: u32,
        ambient_light_distance: f32,
    ) -> Self {
        Self {
            shadow_texture_width,
            shadow_texture_height,
            ambient_light_distance,
        }
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
        // TODO: get current scene center from somewhere (need to find the best place)
        let scene = Vec3::new(0.0, 0.0, 0.0);
        let mut number_of_lights = 0;
        let light_buffer = gpu.extract(&buffers.light);
        let mut shadows = Vec::with_capacity(4);

        for (light,) in world.query::<(&Light,)>() {
            if light.enabled {
                let uniform = light.to_uniform(
                    scene,
                    self.shadow_texture_width,
                    self.shadow_texture_height,
                    self.ambient_light_distance,
                );
                gpu.write_buffer(
                    light_buffer,
                    (std::mem::size_of::<Uniform>() as u32 * number_of_lights) as u64,
                    bytemuck::cast_slice(&[uniform]),
                );
                if light.shadow {
                    shadows.push(number_of_lights);
                }
                number_of_lights += 1;
            }
        }

        Data {
            number_of_lights,
            shadows,
        }
    }
}
