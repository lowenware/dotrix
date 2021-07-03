use dotrix_core::{ Id };
use dotrix_core::assets::{ Texture, Mesh };

use dotrix_math::{ Vec3, InnerSpace };

use crate::{ Heightmap, Generator };

/// Terrain manager (configuration)
pub struct Terrain {
    /// How far the terrain chunks should be spawned (default 500.0)
    pub view_distance: f32,
    /// The lowest lod number (default 4)
    pub max_lod: usize,
    /// Number of polygons per chunk side (default 240)
    pub tile_size: usize,
    /// Terrain will be recalclated only if viewer has moved by that value (default 16*16=256)
    pub spawn_if_moved_by: f32,
    /// Flag to perform force terrain recalculation
    pub force_spawn: bool,
    /// Heights source
    pub heightmap: Box<dyn Heightmap>,
    /// Id of the terrain for texturing
    pub texture: Id<Texture>,
    /// List of the terrain heights to determine UV of the texture
    pub texture_heights: Vec<f32>,
}

impl Terrain {
    /// Constructs new terrain manager
    pub fn new(heightmap: Box<dyn Heightmap>, texture_heights: Vec<f32>) -> Self {
        Self {
            view_distance: 500.0,
            max_lod: 4,
            tile_size: 240,
            spawn_if_moved_by: 256.0,
            force_spawn: true,
            heightmap,
            texture: Id::default(),
            texture_heights,
        }
    }

    /// Generates terrain mesh
    pub fn generate_tile_mesh(&self, tile_x: i32, tile_z: i32, lod: usize) -> Mesh {
        let tile_size = self.tile_size;
        let vertices_per_side = tile_size + 1;
        let offset = self.tile_size as i32 / 2;
        let scale = 2_i32.pow(lod as u32);

        let capacity = vertices_per_side * vertices_per_side;
        let mut positions = Vec::with_capacity(capacity);
        let mut uvs = Vec::with_capacity(capacity);
        let mut normals = vec![[0.0, 0.0, 0.0]; capacity];
        let mut indices = Vec::with_capacity(3 * 2 * self.tile_size * self.tile_size);
        let half_world_size = ((self.heightmap.size() - 1) / 2) as i32;

        for z in -offset..=offset {
            let world_z = tile_z + z * scale;
            let map_z = if world_z < -half_world_size { 0 } else { world_z + half_world_size };
            for x in -offset..=offset {
                let world_x = tile_x + x * scale;
                let map_x = if world_x < -half_world_size { 0 } else { world_x + half_world_size};
                let world_y = self.heightmap.value(map_x as usize, map_z as usize);
                positions.push([world_x as f32, world_y, world_z as f32]);
                uvs.push([
                    (x + offset) as f32 / 2.0 / offset as f32,
                    (z + offset) as f32 / 2.0 / offset as f32,
                ]);
            }
        }

        for z in 0..tile_size {
            let i = (z * vertices_per_side) as u32;
            for x in 0..tile_size {
                let i00 = i + x as u32;
                let i10 = i00 + 1;
                let i01 = i00 + vertices_per_side as u32;
                let i11 = i01 + 1;

                indices.push(i10);
                indices.push(i00);
                indices.push(i01);
                indices.push(i10);
                indices.push(i01);
                indices.push(i11);
            }
        }
        let indices_count = indices.len();
        for i in (0..indices_count).step_by(3) {
            let i0 = indices[i] as usize;
            let i1 = indices[i + 1] as usize;
            let i2 = indices[i + 2] as usize;
            // get the face
            let p0 = Vec3::from(positions[i0]);
            let p1 = Vec3::from(positions[i1]);
            let p2 = Vec3::from(positions[i2]);

            let n1 = p1 - p0;
            let n2 = p2 - p0;
            let normal = n1.cross(n2).normalize();

            normals[i0] = (Vec3::from(normals[i0]) + normal).into();
            normals[i1] = (Vec3::from(normals[i1]) + normal).into();
            normals[i2] = (Vec3::from(normals[i2]) + normal).into();
        }

        for normal in normals.iter_mut() {
            let normalized = Vec3::from(*normal).normalize();
            normal[0] = normalized.x;
            normal[1] = normalized.y;
            normal[2] = normalized.z;
        }

        let mut mesh = Mesh::default();
        mesh.with_vertices(&positions);
        mesh.with_vertices(&normals);
        mesh.with_vertices(&uvs);
        mesh.with_indices(&indices);

        mesh
    }

    /// Calculates texture UV for specific height value
    pub fn uv_from_height(&self, height: f32) -> [f32; 2] {
        let mut i = 0.0;
        for (idx, &tx_height) in self.texture_heights.iter().enumerate() {
            if height > tx_height {
                i = idx as f32;
            } else {
                break;
            }
        }
        let value = i / self.texture_heights.len() as f32;
        [value, value]
    }
}

impl Default for Terrain {
    fn default() -> Self {
        let texture_heights = vec![-1024.0, -128.0, -100.0, 0.0, 32.0];
        Self::new(Box::new(Generator::default()), texture_heights)
    }
}
