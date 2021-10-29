use std::collections::HashMap;
use std::hash::{ Hash, Hasher };
use noise::{ NoiseFn, Perlin };
use rand::{ SeedableRng, RngCore };
use rand::rngs::SmallRng;

use dotrix_core::{ Assets, Id, World };
use dotrix_core::ecs::Entity;
use dotrix_core::assets::Mesh;
use dotrix_core::ray::Ray;
use dotrix_math::Vec3;

use crate::{ Generator, Simple };

/// 2D vector using X and Z axis
pub struct VecXZ<T> {
    pub x: T,
    pub z: T,
}

impl<T> VecXZ<T> {
    pub fn new(x: T, z: T) -> Self {
        Self {
            x,
            z,
        }
    }
}

impl PartialEq for VecXZ<i32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.z == other.z
    }
}

impl Eq for VecXZ<i32> {}

impl Hash for VecXZ<i32> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.z.hash(state);
    }
}

impl Default for VecXZ<i32> {
    fn default() -> Self {
        Self {
            x: 0,
            z: 0,
        }
    }
}

impl<T: Copy + Clone> Copy for VecXZ<T> { }

impl<T: Copy + Clone> Clone for VecXZ<T> {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            z: self.z,
        }
    }
}

/// Component represents single sqare terrain mesh.
/// This enumeration defines number of vertices and units of the mesh.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Component {
    /// Tiny mesh with 63 vertices per side (for tiny game worlds)
    Tiny,
    /// Small mesh with 127 vertices per side (for small game worlds)
    Small,
    /// Standard mesh with 255 vertices per side (for big game worlds)
    Standard,
}

// Note: The Component can be reused for voxel based terrain as well,
// by defining more methods or introducing traits
impl Component {
    /// Returns number of vertices per side (always odd)
    pub fn vertices_per_side(self) -> usize {
        match self {
            Component::Tiny => 63,
            Component::Small => 127,
            Component::Standard => 255,
        }
    }

    /// Returns number of units per side (always even)
    pub fn units_per_side(self) -> usize {
        self.vertices_per_side() - 1
    }

    /// Returns axis offset of component's position on the map
    /// Component is identified by the Position, which is in fact a position of its center
    pub fn axis_offset(axis: i32, lod: Lod) -> i32 {
        lod.scale() as i32 * (2 * axis + 1)
    }
}

/// Terrain Level of Details
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Lod {
    scale: u32,
}

impl Lod {
    /// Creates Lod instance from level number (0 is the highest detalization)
    pub fn from_level(level: u32) -> Self {
        Self {
            scale: (2_u32).pow(level)
        }
    }

    /// Returns component scale for specified Lod
    pub fn scale(&self) -> u32 {
        self.scale
    }
}

impl Default for Lod {
    fn default() -> Self {
        Self::from_level(0)
    }
}

/// Terrain Node
/// Stores information about spawned component for further manipulations
#[derive(Default)]
pub struct Node {
    /// Level of Details of the Node
    pub lod: Lod,
    /// Cleanup request flag
    pub cleanup: bool,
    /// Spawn state flag
    pub spawned: bool,
    /// True value means, that terrain node should be fully regenerated
    pub dirty: bool,
    /// Id of the node terrain Mesh
    pub mesh: Id<Mesh>,
    pub entity: Option<Entity>,
}

/// World Terrain Map service
pub struct Map {
    /// Component type
    pub component: Component,
    /// Unit size (Dotrix Unit can be interpreted as 1m in game world)
    pub unit_size: f32,
    /// Minimal level of details
    pub min_lod: Lod,
    /// Terrain view range (number of components)
    pub view_range: u32,
    /// Terrain offset in game world (to center the terrain in world coordinates)
    pub offset: VecXZ<f32>,
    /// Terrain nodes map
    pub nodes: HashMap<VecXZ<i32>, Node>,
    /// Mesh backer
    pub generator: Box<dyn Generator>,
}

impl Map {
    pub fn component_size(&self, lod: Lod) -> f32 {
        self.component.units_per_side() as f32 * self.unit_size * lod.scale() as f32
    }

    pub fn request_cleanup(&mut self) {
        for node in self.nodes.values_mut() {
            node.cleanup = true;
        }
    }

    pub fn intersection(&self, ray: &Ray, range: f32) -> Option<Vec3> {
        self.generator.intersection(ray, range, self.unit_size)
    }

    pub fn modify(&mut self, point: &Vec3, values: &[f32], size: u32) {
        self.generator.modify(point, values, size, self.unit_size);
    }

    pub fn flatten(&mut self, point: &Vec3, values: &[f32], size: u32) {
        self.generator.flatten(point, values, size, self.unit_size);
    }

    pub fn set_tile_dirty(&mut self, point: &Vec3, size: u32) {
        let radius = (size as f32 / 2.0) * self.unit_size;
        let dirty_from = VecXZ::new(point.x - radius, point.z - radius);
        let dirty_to = VecXZ::new(point.x + radius, point.z + radius);
        let scale = (self.component.units_per_side() / 2) as f32 * self.unit_size;
        for (pos, node) in self.nodes.iter_mut() {
            let pos_x0 = (pos.x - node.lod.scale() as i32) as f32 * scale;
            let pos_x1 = (pos.x + node.lod.scale() as i32) as f32 * scale;
            let pos_z0 = (pos.z - node.lod.scale() as i32) as f32 * scale;
            let pos_z1 = (pos.z + node.lod.scale() as i32) as f32 * scale;

            if pos_x0 >= dirty_to.x || dirty_from.x >= pos_x1 {
                continue;
            }

            if (dirty_from.z >= pos_z1 || pos_z0 >= dirty_to.z) {
                continue;
            }

            node.dirty = true;
        }
    }

    pub fn set_dirty(&mut self) {
        for (_, node) in self.nodes.iter_mut() {
            node.dirty = true;
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        Self {
            component: Component::Standard,
            unit_size: 1.0,
            min_lod: Lod::from_level(3),
            view_range: 2,
            offset: VecXZ::new(0.0, 0.0),
            nodes: HashMap::new(),
            generator: Box::new(Simple::default()),
        }
    }
}

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

        let octaves_offsets = (0..self.octaves).map(|_| {
            max_noise_height += amplitude;
            amplitude *= self.persistence;

            [
                Self::randomize_offset(self.offset[0], &mut pseudo_rng),
                Self::randomize_offset(self.offset[1], &mut pseudo_rng)
            ]
        }).collect::<Vec<_>>();

        let mut min_noise_height = 0.0;
        let mut max_noise_height = 0.0;

        let half_size = (size / 2) as f32;
        for x in 0..size {
            for z in 0..size {
                let mut noise_height = 0.0;

                amplitude = 1.0;
                frequency = 1.0;

                for octave_offset in octaves_offsets.iter() {
                    let xf = (x as f32 - half_size + octave_offset[0])
                        / self.scale
                        * frequency;
                    let zf = (z as f32 - half_size + octave_offset[1])
                        / self.scale
                        * frequency;

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
        Self { power: 2.6, factor: 2.4 }
    }
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
	}
}
