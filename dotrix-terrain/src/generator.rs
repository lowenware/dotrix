use crate::Heightmap;
use noise::{NoiseFn, Perlin};

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

/// Noise configuration
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Noise {
    /// Noise frequency
    pub frequency: f32,
    /// Number of octaves
    pub octaves: usize,
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

impl Noise {
    /// Returns noise map of values
    pub fn map(&self, size: usize) -> Vec<f32> {
        let mut map = Vec::with_capacity(size * size);
        let noise = Perlin::new();

        let mut max_noise_height = 0.0;
        let mut amplitude = 1.0;
        let mut frequency;

        let mut pseudo_rng = SmallRng::seed_from_u64(self.seed as u64);

        let octaves_offsets = (0..self.octaves)
            .map(|_| {
                max_noise_height += amplitude;
                amplitude *= self.persistence;

                [
                    Self::randomize_offset(self.offset[0], &mut pseudo_rng),
                    Self::randomize_offset(self.offset[1], &mut pseudo_rng),
                ]
            })
            .collect::<Vec<_>>();

        let mut min_noise_height = 0.0;
        let mut max_noise_height = 0.0;

        let half_size = (size / 2) as f32;
        for x in 0..size {
            for z in 0..size {
                let mut noise_height = 0.0;

                amplitude = 1.0;
                frequency = 1.0;

                for octave_offset in octaves_offsets.iter() {
                    let xf = (x as f32 - half_size + octave_offset[0]) / self.scale * frequency;
                    let zf = (z as f32 - half_size + octave_offset[1]) / self.scale * frequency;

                    let noise_value = noise.get([xf as f64, zf as f64]) as f32; // (-1..1);
                    noise_height += noise_value * amplitude;

                    amplitude *= self.persistence;
                    frequency *= self.lacunarity;
                }

                if noise_height < min_noise_height {
                    min_noise_height = noise_height;
                }

                if noise_height > max_noise_height {
                    max_noise_height = noise_height;
                }

                map.push(noise_height as f32);
            }
        }

        // normalize values
        let delta = max_noise_height - min_noise_height;
        let offset = min_noise_height + delta / 2.0;

        for m in &mut map {
            *m = *m / delta - offset;
        }

        map
    }

    fn randomize_offset(value: f32, pseudo_rng: &mut SmallRng) -> f32 {
        value + (pseudo_rng.next_u32() & 0xFFFF) as f32 - 32768.0
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            frequency: 1.1,
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
#[derive(Clone, Copy)]
pub struct Falloff {
    /// How rough should fall off
    pub power: f32,
    /// How far should fall off
    pub factor: f32,
}

impl Falloff {
    /// Returns falloff map
    pub fn map(&self, size: usize) -> Vec<f32> {
        let mut map = Vec::with_capacity(size * size);
        for x in 0..size {
            for z in 0..size {
                let value_x = (x as f32 / size as f32 * 2.0 - 1.0).abs();
                let value_z = (z as f32 / size as f32 * 2.0 - 1.0).abs();
                let value = if value_x > value_z { value_x } else { value_z };
                let value = self.evaluate(value);
                map.push(value);
            }
        }
        map
    }

    #[inline(always)]
    fn evaluate(&self, value: f32) -> f32 {
        let power_of_value = value.powf(self.power);
        power_of_value / (power_of_value + (self.factor - self.factor * value).powf(self.power))
    }
}

impl Default for Falloff {
    fn default() -> Self {
        Self {
            power: 2.6,
            factor: 2.4,
        }
    }
}

/// Terrain heights generator from Pelin noise
#[derive(Default)]
pub struct Generator {
    /// Amplitude of the heights generation
    pub amplitude: f32,
    /// Size of the heightmap
    pub size: usize,
    /// Noisemap values
    pub noise_map: Option<Vec<f32>>,
    /// Falloff values
    pub falloff_map: Option<Vec<f32>>,
}

impl Heightmap for Generator {
    fn value(&self, x: usize, z: usize) -> f32 {
        self.noise_map
            .as_ref()
            .map(|noise_map| {
                let i = x * self.size + z;
                let mut value = if i < noise_map.len() {
                    noise_map[i]
                } else {
                    0.0
                };
                if let Some(falloff_map) = self.falloff_map.as_ref() {
                    if i < falloff_map.len() {
                        value -= falloff_map[i];
                    }
                }
                self.amplitude * num::clamp(value, 0.0, 1.0)
            })
            .unwrap_or(0.0)
    }

    fn size(&self) -> usize {
        self.size
    }
}
