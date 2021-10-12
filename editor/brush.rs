use dotrix::prelude::*;
use dotrix::{ Assets, Input };
use dotrix::assets::Texture;
use dotrix::input::{ Button, State as InputState };
use dotrix::ray::Ray;
use dotrix::terrain::{ Map, HeightMap };
use dotrix::math::Vec3;


pub const BRUSH_TEXTURE: &str = "dotrix::editor::brush";

pub struct Brush {
    radius: u32,
    data: Vec<f32>,
}

impl Brush {

    pub fn radial(radius: u32) -> Self {
        let side = 2 * radius;
        let size = (side * side) as usize;
        let mut data = Vec::with_capacity(size);
        let alpha = Self::distance(0, radius, radius, radius);

        for u in 0..side {
            for v in 0..side {
                let value = (alpha - Self::distance(u, v, radius, radius)) / alpha;
                data.push(if value > 0.0 { value } else { 0.0 });
            }
        }

        Self {
            radius,
            data
        }
    }

    fn distance(u1: u32, v1: u32, u2: u32, v2: u32) -> f32 {
        let du = u1 as i32 - u2 as i32;
        let dv = v1 as i32 - v2 as i32;
        ((du * du + dv * dv) as f32).sqrt()
    }

    pub fn texture(&self) -> Texture {
        let bytes_per_pixel = 4;
        let mut data = Vec::with_capacity(self.data.len() * bytes_per_pixel);
        let max_value: u8 = 0xFF;
        let size = self.radius * 2;
        for value in self.data.iter() {
            let byte = (max_value as f32 * value) as u8;
            data.push(byte); // R
            data.push(byte); // G
            data.push(byte); // B
            data.push(max_value); // A
        }
        Texture {
            width: size,
            height: size,
            data,
            ..Default::default()
        }
    }
}

pub fn startup(
    mut assets: Mut<Assets>,
) {
    let brush = Brush::radial(256);
    assets.store_as(brush.texture(), BRUSH_TEXTURE);
}

pub fn update(
    ray: Const<Ray>,
    input: Const<Input>,
    mut map: Mut<Map>,
) {
    if input.button_state(Button::MouseLeft) != Some(InputState::Hold) {
        return;
    }

    /*
    if let Some(intersection) = terrain.ray_intersection(&ray) {
        if let Some(heightmap) = terrain.heightmap_mut::<HeightMap>() {
            let size = heightmap.size() as f32;
            let offset = size as f32 / 2.0;
            let x = intersection.x + offset;
            let z = intersection.z + offset;

            if x < 0.0 || x >= size || z < 0.0 || z >= size {
                return;
            }

            let x = x.round() as usize;
            let z = z.round() as usize;

            if let Some(value) = heightmap.get(x, z) {
                heightmap.set(x, z, value + 256);
                terrain.force_spawn = true;
            }
        }
    }
    */
}

