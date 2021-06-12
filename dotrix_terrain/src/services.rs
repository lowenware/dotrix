use crate::{ Heightmap, Generator, Terrain };
use dotrix_core::{
    assets::{Id, Texture, Mesh},
};
use dotrix_math::{Vec3, InnerSpace};

// Consider name it a Landmass
/// Terrain manager (configuration)
pub struct Manager {
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

impl Manager {
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
            texture_heights
        }
    }

    /// Generates terrain mesh
    pub fn generate_mesh(&self, terrain: &Terrain) -> Mesh {
        let tile_size = self.tile_size;
        let vertices_per_side = tile_size + 1;
        let offset = self.tile_size as i32 / 2;
        let scale = 2_i32.pow(terrain.lod as u32);

        let capacity = vertices_per_side * vertices_per_side;
        let mut positions = Vec::with_capacity(capacity);
        let mut uvs = Vec::with_capacity(capacity);
        let mut normals = vec![[0.0, 0.0, 0.0]; capacity];
        let mut indices = Vec::with_capacity(3 * 2 * self.tile_size * self.tile_size);

        let mut min_yf = 0.0;
        let mut max_yf = 0.0;

        for z in -offset..=offset {
            let zf = (terrain.z + z * scale) as f32;
            for x in -offset..=offset {
                let xf = (terrain.x + x * scale) as f32;
                let yf = self.heightmap.y_value(xf, zf);
                if min_yf > yf { min_yf = yf; }
                if max_yf < yf { max_yf = yf; }
                positions.push([xf, yf, zf]);
                uvs.push(self.uv_from_height(yf));
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

        Mesh {
            positions,
            normals: Some(normals),
            uvs: Some(uvs),
            indices: Some(indices),
            ..Default::default()
        }
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

impl Default for Manager {
    fn default() -> Self {
        let texture_heights = vec![-1024.0, -128.0, -100.0, 0.0, 32.0];
        Self::new(Box::new(Generator::default()), texture_heights)
    }
}
