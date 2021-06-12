use noise::{ NoiseFn, Fbm, MultiFractal };

/// Trait for the terrain heights source
pub trait Heightmap: Sync + Send {
    /// Returns Y axis value for specified X and Z pair
    fn y_value(&self, x: f32, z: f32) -> f32;
}

/// Terrain heights generator from Pelin noise
pub struct Generator {
    /// Noise amplitude multiplier
    pub amplitude: f32,
    /// Noise scale
    pub scale: f32,
    /// Noise instance
    pub noise: Fbm,
}

impl Default for Generator {
    fn default() -> Self {

        let octaves = 8;
        let frequency = 1.1;
        let lacunarity = 4.5;
        let persistence = 0.1;

        let noise = Fbm::new();
        let noise = noise.set_octaves(octaves);
        let noise = noise.set_frequency(frequency);
        let noise = noise.set_lacunarity(lacunarity);
        let noise = noise.set_persistence(persistence);

        Self {
            scale: 512.0,
            amplitude: 512.0,
            noise,
        }
    }
}

impl Heightmap for Generator {
    fn y_value(&self, x: f32, z: f32) -> f32 {
        let scale = self.scale;
        let sample_x = x / scale;
        let sample_z = z / scale;
        self.amplitude * self.noise.get([sample_x as f64, sample_z as f64]) as f32
    }
}
