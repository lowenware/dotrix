use std::collections::HashMap;
use dotrix::prelude::*;
use dotrix::{ Assets, Camera, Color, World };
use dotrix::assets::Texture;
use dotrix::pbr::Light;
use dotrix::math::Vec3;
use dotrix::terrain::{ Map, HeightMap, Lod, Simple as SimpleTerrain };

pub use dotrix::terrain::extension;

const COMPONENT_SIZE: usize = 128;

#[derive(Hash, Debug)]
pub struct VecXZ {
    x: u32,
    z: u32,
}

#[derive(Debug, Copy, Clone)]
struct Node {
    is_dirty: bool
}

impl Default for Node {
    fn default() -> Self {
        Self {
            is_dirty: true
        }
    }
}

/*

pub struct Map {
    scale: u32,
    nodes: Vec<Vec<Node>>,
    heights: Texture,
}

impl Map {
    pub fn new(heights: Texture, scale: u32) -> Self {
        let mut map = Self {
            scale,
            nodes: Vec::new(),
            heights,
        };
        map.reallocate();
        map
    }

    pub fn resize(&mut self, new_size_x: usize, new_size_z: usize) {
        let old_size_x = self.heights.width as usize;
        let old_size_z = self.heights.height as usize;
        let old_bytes = self.heights.data.len();

        let bytes_per_pixel = old_bytes / old_size_x / old_size_z;
        let new_bytes = new_size_x * new_size_z * bytes_per_pixel;

        // Extend the array to move data
        if new_bytes > old_bytes {
            self.heights.data.resize(new_bytes, 0);
        }

        for z in (0..new_size_z).rev() {
            for x in (0..new_size_x).rev() {
                let new_index = bytes_per_pixel * (z * new_size_x + x);
                let old_index = bytes_per_pixel * (z * old_size_x + x);
                let use_old_value = x < old_size_x && z < old_size_z;
                for byte in 0..bytes_per_pixel {
                    self.heights.data[new_index + byte] = if use_old_value {
                        self.heights.data[old_index + byte]
                    } else {
                        0
                    };
                }
            }
        }

        // Shrink array if it was necessary
        if new_bytes < old_bytes {
            self.heights.data.resize(new_bytes as usize, 0);
        }

        self.heights.width = new_size_x as u32;
        self.heights.height = new_size_z as u32;
        self.reallocate();
    }

    fn reallocate(&mut self) {
        let size_x = self.heights.width;
        let size_z = self.heights.height;
        let nodes_per_x = (size_x as f32 / COMPONENT_SIZE as f32).ceil() as usize;
        let nodes_per_z = (size_z as f32 / COMPONENT_SIZE as f32).ceil() as usize;
        let world_offset_x = size_x / 2;
        let world_offset_z = size_z / 2;

        for list in self.nodes.iter_mut() {
            let len = list.len();
            let last = if nodes_per_z < len { nodes_per_z } else { len } - 1;
            list[last].is_dirty = true;

            if nodes_per_z != len {
                list.resize_with(nodes_per_z, Node::default)
            }
        }

        if !self.nodes.is_empty() {
            let len = self.nodes.len();
            let last = if nodes_per_x < len { nodes_per_x } else { len } - 1;
            for node in self.nodes[last].iter_mut() {
                node.is_dirty = true;
            }
        }

        self.nodes.resize_with(nodes_per_x, || vec![Node::default(); nodes_per_z]);
    }
}

impl Default for Map {
    fn default() -> Self {
        let bytes_per_pixel = 2;
        let size = 1024;
        let texture = Texture {
            width: size,
            height: size,
            data: vec![0; (bytes_per_pixel * size * size) as usize],
            ..Default::default()
        };
        let scale = 4;
        Self::new(texture, scale)
    }
}

*/


pub fn startup(
    mut assets: Mut<Assets>,
    mut map: Mut<Map>,
) {
    let texture = assets.store(Texture {
        width: 1,
        height: 1,
        data: vec![0xAE, 0xAE, 0xAE, 0xFF],
        ..Default::default()
    });

    println!("Logo texture: {:?}", assets.register::<Texture>("lowenware"));

    assets.import("assets/lowenware.png");
    assets.import("assets/terrain/heightmap.png");

    // terrain.heightmap = Box::new(HeightMap::new(8129));
    // map.texture = texture;
    map.min_lod = Lod::from_level(3);
    map.unit_size = 1.0;
}

#[derive(Default)]
pub struct Loader {
    loaded: bool,
}

pub fn load(
    mut context: Context<Loader>,
    mut assets: Mut<Assets>,
    mut map: Mut<Map>,
) {
    if !context.loaded {
        let texture_id = assets.register::<Texture>("heightmap");
        if let Some(texture) = assets.remove(texture_id) {
            println!("HeightMap has been loaded");
            let height_map = HeightMap::from(texture);
            let mut terrain = SimpleTerrain::from(height_map);
            terrain.max_height = 640.0;
            map.generator = Box::new(terrain);
            context.loaded = true;
        }
    }
}

