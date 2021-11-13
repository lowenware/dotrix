use noise::{ NoiseFn, Perlin };
use crate::Heightmap;

use rand::{ SeedableRng, RngCore };
use rand::rngs::SmallRng;

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
                let mut value = if i < noise_map.len() { noise_map[i] } else { 0.0 };
                if let Some(falloff_map) = self.falloff_map.as_ref() {
                    if i < falloff_map.len() {
                        value -= falloff_map[i];
                    }
                }
                num::clamp(value, 0.0, 1.0)
            })
            .unwrap_or(0.0)
    }

    fn size(&self) -> usize {
        self.size
    }
}
