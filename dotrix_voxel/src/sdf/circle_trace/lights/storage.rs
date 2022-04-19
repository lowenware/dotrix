// Originally from https://github.com/lowenware/dotrix/blob/4582922bbdd8bd039857271f8c8eb4cfd42fcb00/dotrix_pbr/src/light.rs
//
use super::data::*;
use dotrix_core::{
    ecs::{Const, Mut},
    renderer::Buffer,
    Globals, Renderer, World,
};

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub(super) struct GenericLight {
    pub(super) position: [f32; 4],
    pub(super) direction: [f32; 4],
    pub(super) color: [f32; 4],
    pub(super) parameters: [f32; 4],
    pub(super) kind: u32, // 1 = DirectionalLight, 2 = PointLight, 3 = SimpleLight, 4 = SpotLight, 0 = None
    pub(super) padding: [f32; 3],
}

unsafe impl bytemuck::Zeroable for GenericLight {}
unsafe impl bytemuck::Pod for GenericLight {}

pub struct LightStorageBuffer {
    pub storage: Buffer,
}

impl Default for LightStorageBuffer {
    fn default() -> Self {
        Self {
            storage: Buffer::storage("Light Storage"),
        }
    }
}

pub fn startup(mut globals: Mut<Globals>) {
    globals.set(LightStorageBuffer::default());
}

/// Lights binding system
pub fn load(world: Const<World>, renderer: Const<Renderer>, mut globals: Mut<Globals>) {
    if let Some(lights) = globals.get_mut::<LightStorageBuffer>() {
        let mut generic_lights: Vec<GenericLight> = world
            .query::<(&Light,)>()
            .flat_map(|(light,)| match light {
                Light::Directional {
                    color,
                    direction,
                    intensity,
                    enabled,
                } if *enabled => Some(
                    DirectionalLight {
                        direction: [direction.x, direction.y, direction.z, 1.0],
                        color: (*color * (*intensity)).into(),
                    }
                    .into(),
                ),
                Light::Point {
                    color,
                    position,
                    intensity,
                    enabled,
                    constant,
                    linear,
                    quadratic,
                } if *enabled => Some(
                    PointLight {
                        position: [position.x, position.y, position.z, 1.0],
                        color: (*color * (*intensity)).into(),
                        a_constant: *constant,
                        a_linear: *linear,
                        a_quadratic: *quadratic,
                    }
                    .into(),
                ),
                Light::Simple {
                    color,
                    position,
                    intensity,
                    enabled,
                } if *enabled => Some(
                    SimpleLight {
                        position: [position.x, position.y, position.z, 1.0],
                        color: (*color * (*intensity)).into(),
                    }
                    .into(),
                ),
                Light::Spot {
                    color,
                    position,
                    direction,
                    intensity,
                    enabled,
                    cut_off,
                    outer_cut_off,
                } if *enabled => Some(
                    SpotLight {
                        position: [position.x, position.y, position.z, 1.0],
                        direction: [direction.x, direction.y, direction.z, 1.0],
                        color: (*color * (*intensity)).into(),
                        cut_off: *cut_off,
                        outer_cut_off: *outer_cut_off,
                    }
                    .into(),
                ),
                _ => None,
            })
            .collect();

        let ambient: GenericLight = world
            .query::<(&Light,)>()
            .flat_map(|(light,)| match light {
                Light::Ambient { color, intensity } => Some(GenericLight {
                    color: (*color * (*intensity)).into(),
                    ..Default::default()
                }),
                _ => None,
            })
            .fold(GenericLight::default(), |mut ambient, light| {
                ambient.color[0] += light.color[0];
                ambient.color[1] += light.color[1];
                ambient.color[2] += light.color[2];
                ambient.color[3] += light.color[3];
                ambient
            });

        generic_lights.push(ambient);

        renderer.load_buffer(
            &mut lights.storage,
            bytemuck::cast_slice(generic_lights.as_slice()),
        );
    }
}
