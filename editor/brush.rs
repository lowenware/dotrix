use dotrix::prelude::*;
use dotrix::{ Assets, Input };
use dotrix::assets::Texture;
use dotrix::input::{ Button, State as InputState };
use dotrix::ray::Ray;
use dotrix::terrain::{ Map, HeightMap };
use dotrix::math::Vec3;


pub const BRUSH_TEXTURE: &str = "dotrix::editor::brush";

pub struct Brush {
    size: u32,
    values: Vec<f32>,
}

impl Brush {
    pub fn radial(size: u32, intensity: f32) -> Self {
        let capacity = (size * size) as usize;
        let mut values = Vec::with_capacity(capacity);
        let radius = size / 2;
        let alpha = Self::distance(0, radius, radius, radius);

        for u in 0..size {
            for v in 0..size {
                let value = alpha - Self::distance(u, v, radius, radius);
                values.push(if value > 0.0 { intensity * value / alpha } else { 0.0 });
            }
        }

        Self {
            size,
            values
        }
    }

    fn distance(u1: u32, v1: u32, u2: u32, v2: u32) -> f32 {
        let du = u1 as i32 - u2 as i32;
        let dv = v1 as i32 - v2 as i32;
        ((du * du + dv * dv) as f32).sqrt()
    }

    pub fn texture(&self) -> Texture {
        let bytes_per_pixel = 4;
        let mut data = Vec::with_capacity(self.values.len() * bytes_per_pixel);
        let max_value: u8 = 0xFF;
        let size = self.size;
        for value in self.values.iter() {
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

impl Default for Brush {
    fn default() -> Self {
        Self::radial(512, 0.5)
    }
}

pub fn startup(
    mut assets: Mut<Assets>,
    brush: Const<Brush>,
) {
    assets.store_as(brush.texture(), BRUSH_TEXTURE);
}

pub fn update(
    mut map: Mut<Map>,
    ray: Const<Ray>,
    input: Const<Input>,
    brush: Const<Brush>,
) {
    let range: f32 = 64000.0;
    if input.button_state(Button::MouseLeft) != Some(InputState::Hold) {
        return;
    }

    if let Some(point) = map.intersection(&ray, range) {
        println!("Ray intersects terrain @ {:?}", point);
        map.modify(&point, &brush.values, brush.size);
        map.set_dirty(&point, brush.size);
        /*
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
        */
    }
}

