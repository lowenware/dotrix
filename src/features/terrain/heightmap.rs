use noise::{NoiseFn, Perlin};
use serde::{Deserialize, Serialize};

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

use crate::Asset;

pub struct HeightMap {
    name: String,
    values: Vec<f32>,
    size: u32,
}

impl HeightMap {
    pub fn new(name: impl Into<String>, size: u32) -> Self {
        Self {
            name: name.into(),
            values: vec![0.0; (size * size) as usize],
            size,
        }
    }

    pub fn new_from_noise(name: impl Into<String>, size: u32, noise_config: &NoiseConfig) -> Self {
        let mut noise_values = vec![0.0; (size * size) as usize];
        let noise = Perlin::new(noise_config.seed);

        let mut pseudo_rng = SmallRng::seed_from_u64(noise_config.seed as u64);

        let octaves_offsets = (0..noise_config.octaves)
            .map(|_| {
                [
                    Self::randomize_offset(noise_config.offset[0], &mut pseudo_rng),
                    Self::randomize_offset(noise_config.offset[1], &mut pseudo_rng),
                ]
            })
            .collect::<Vec<_>>();

        let mut min_noise_value: f32 = f32::MAX;
        let mut max_noise_value: f32 = f32::MIN;

        let half_size = (size / 2) as f32; // to apply scale to the center
        for z in 0..size {
            let offset = z * size;
            for x in 0..size {
                let index = (offset + x) as usize;
                let mut noise_value = 0.0;

                let mut amplitude = 1.0;
                let mut frequency = 1.0;

                for octave_offset in octaves_offsets.iter() {
                    let xf =
                        (x as f32 - half_size) / noise_config.scale * frequency + octave_offset[0];
                    let zf =
                        (z as f32 - half_size) / noise_config.scale * frequency + octave_offset[1];

                    let noise_octave_value = 2.0 * (noise.get([xf as f64, zf as f64]) as f32) - 1.0;
                    noise_value += noise_octave_value * amplitude;

                    amplitude *= noise_config.persistence;
                    frequency *= noise_config.lacunarity;
                }

                if noise_value < min_noise_value {
                    min_noise_value = noise_value;
                }

                if noise_value > max_noise_value {
                    max_noise_value = noise_value;
                }

                noise_values[index] = noise_value;
            }
        }

        // normalize values
        let delta = max_noise_value - min_noise_value;
        let values = noise_values
            .into_iter()
            .map(|value| (((value - min_noise_value) / delta).clamp(0.0, 1.0)))
            .collect::<Vec<_>>();

        Self {
            name: name.into(),
            values,
            size,
        }
    }

    /// Returns falloff map
    pub fn new_from_falloff(
        name: impl Into<String>,
        size: u32,
        falloff_config: &FalloffConfig,
    ) -> Self {
        let mut values = vec![0.0; (size * size) as usize];
        for z in 0..size {
            let offset = z * size;
            for x in 0..size {
                let index = (offset + x) as usize;
                let value_x = (x as f32 / size as f32 * 2.0 - 1.0).abs();
                let value_z = (z as f32 / size as f32 * 2.0 - 1.0).abs();
                let value = if value_x > value_z { value_x } else { value_z };
                // soften the curve
                let power_of_value = value.powf(falloff_config.power);
                let value = power_of_value
                    / (power_of_value
                        + (falloff_config.factor - falloff_config.factor * value)
                            .powf(falloff_config.power));

                values[index] = value.clamp(0.0, 1.0);
            }
        }
        Self {
            name: name.into(),
            size,
            values,
        }
    }

    pub fn new_from_bytes(name: impl Into<String>, size: u32, bytes: &[u8]) -> Self {
        let pixels_count = (size * size) as usize;
        let bytes_per_pixel = bytes.len() / pixels_count;
        let mut values = vec![0.0; pixels_count];

        let mut max_value: u32 = 0;
        for _ in 0..bytes_per_pixel {
            max_value = (max_value << 8) | 0xFF;
        }

        //log::debug!("max_value: {}", max_value);

        for i in 0..pixels_count {
            let mut value_uint: u32 = 0;
            for byte in (0..bytes_per_pixel).rev() {
                // value_uint =
                //     value_uint | ((bytes[i * bytes_per_pixel + byte] as u32) << byte_shift);
                value_uint = (value_uint << 8) | (bytes[i * bytes_per_pixel + byte] as u32);
            }
            // log::debug!("value_uint: 0x{:.04x}", value_uint);
            values[i] = (value_uint as f64 / max_value as f64) as f32;
        }
        // panic!("brea");
        Self {
            name: name.into(),
            values,
            size,
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn value(&self, x: u32, z: u32) -> f32 {
        let i = (z * self.size + x) as usize;
        self.values[i]
    }

    pub fn subtract(&mut self, map: &Self) {
        if self.size != map.size {
            panic!(
                "Could not subtract maps of different sizes: {} != {}",
                self.size, map.size,
            );
        }

        for z in 0..self.size {
            let offset = z * self.size;
            for x in 0..self.size {
                let i = (offset + x) as usize;
                self.values[i] = (self.values[i] - map.values[i]).clamp(0.0, 1.0);
            }
        }
    }

    fn randomize_offset(value: f32, pseudo_rng: &mut SmallRng) -> f32 {
        value + (pseudo_rng.next_u32() & 0xFFFF) as f32 - 32768.0
    }

    pub fn write_to_file(
        &self,
        path: &std::path::Path,
        format: image::ImageFormat,
    ) -> Result<(), image::ImageError> {
        let size = self.size;
        let buffer = (0..size)
            .flat_map(|z| (0..size).map(move |x| ((self.value(x, z) * 65535.0).round() as u16)))
            .collect::<Vec<_>>();

        let image_buffer: image::ImageBuffer<image::Luma<u16>, Vec<u16>> =
            image::ImageBuffer::from_vec(size, size, buffer)
                .expect("Could not generate heightmap pixels buffers");

        image_buffer
            //image::GrayImage::from_raw(size, size, pixels)
            .save_with_format(path, format)
    }
}

impl Asset for HeightMap {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Noise configuration
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct NoiseConfig {
    /// Number of octaves
    pub octaves: u32,
    /// Noise persistence
    pub persistence: f32,
    /// Noise Lacunarity
    pub lacunarity: f32,
    /// Noise scale
    pub scale: f32,
    /// Offset of the noise sampling
    pub offset: [f32; 2],
    /// Noise seed
    pub seed: u32,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
            scale: 250.0,
            offset: [0.0, 0.0],
            seed: 0,
        }
    }
}

/// Falloff settings
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct FalloffConfig {
    /// How rough should fall off
    pub power: f32,
    /// How far should fall off
    pub factor: f32,
}

impl Default for FalloffConfig {
    fn default() -> Self {
        Self {
            power: 2.6,
            factor: 2.4,
        }
    }
}
