use dotrix::prelude::*;
use dotrix::{ Assets, Input };
use dotrix::assets::Texture;
use dotrix::input::{ Button, State as InputState };
use dotrix::ray::Ray;
use dotrix::terrain::{ Map, HeightMap, Noise };
use dotrix::math::Vec3;


pub const BRUSH_TEXTURE: &str = "dotrix::editor::brush";
pub const INTENSITY: f32 = 0.1;
pub const SIZE: u32 = 512;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Mode {
    Elevate,
    Flatten,
}

pub struct Brush {
    pub mode: Mode,
    pub size: u32,
    pub values: Vec<f32>,
    pub changed: bool,
}

impl Brush {
    pub fn radial(size: u32, intensity: f32) -> Vec<f32> {
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

        values
    }

    pub fn noise(size: u32, intensity: f32, noise: &Noise) -> Vec<f32> {
        let mut map = noise.map(size as usize);
        let radial = Brush::radial(size, intensity);
        for (value, r) in map.iter_mut().zip(radial.iter()) {
            *value *= *r;
        }
        map
    }

    fn distance(u1: u32, v1: u32, u2: u32, v2: u32) -> f32 {
        let du = u1 as i32 - u2 as i32;
        let dv = v1 as i32 - v2 as i32;
        ((du * du + dv * dv) as f32).sqrt()
    }

    pub fn texture(&self) -> Texture {
        let bytes_per_pixel = 4;
        let mut data = Vec::with_capacity(self.values.len() * bytes_per_pixel);
        let max_byte: u8 = 0xFF;
        let size = self.size;
        let mut max_value = -1.0;
        let mut min_value = 1.0;

        for &value in self.values.iter() {
            if value > max_value {
                max_value = value;
            }

            if value < min_value {
                min_value = value;
            }
        }
        let delta = max_value - min_value;
        for value in self.values.iter() {
            let byte = (max_byte as f32 * (value - min_value) / delta) as u8;
            data.push(byte); // R
            data.push(byte); // G
            data.push(byte); // B
            data.push(max_byte); // A
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
        let size = SIZE;
        let intensity = INTENSITY;
        Self {
            mode: Mode::Elevate,
            size,
            values: Self::radial(size, intensity),
            changed: false,
        }
    }
}

pub fn startup(
    mut assets: Mut<Assets>,
    brush: Const<Brush>,
) {
    assets.store_as(brush.texture(), BRUSH_TEXTURE);
}

pub fn update(
    mut assets: Mut<Assets>,
    mut brush: Mut<Brush>,
    mut map: Mut<Map>,
    ray: Const<Ray>,
    input: Const<Input>,
) {
    let range: f32 = 64000.0;

    if brush.changed {
        let new_texture = brush.texture();
        if let Some(texture_id) = assets.find(BRUSH_TEXTURE) {
            if let Some(texture) = assets.get_mut::<Texture>(texture_id) {
                texture.data = new_texture.data;
                texture.width = new_texture.width;
                texture.height = new_texture.height;
                texture.unload();
                brush.changed = false;
            }
        }
    }

    if input.button_state(Button::MouseLeft) != Some(InputState::Hold) {
        return;
    }

    if let Some(point) = map.intersection(&ray, range) {
        println!("Ray intersects terrain @ {:?}", point);
        match brush.mode {
            Mode::Elevate => map.modify(&point, &brush.values, brush.size),
            Mode::Flatten => map.flatten(&point, &brush.values, brush.size),
        };
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

