use dotrix_core::{ Id };
use dotrix_core::ray::Ray;
use dotrix_core::assets::{ Texture, Mesh };

use dotrix_math::{ Vec3, InnerSpace };

use crate::{ GetHeight, Generator };



pub struct VecXZ<T> {
    pub x: T,
    pub z: T,
}

/// Terrain manager (configuration)
pub struct Map {
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
    pub heightmap: Box<dyn GetHeight>,
    /// Id of the terrain for texturing
    pub texture: Id<Texture>,
    /// Maximal terrain height
    pub max_height: f32,
    /// Terrain unit size,
    pub unit_size: f32,
    /// Terrain mesh size
    pub mesh_size: MeshSize,
}

impl Map {
    /// Constructs new terrain manager
    pub fn new(heightmap: Box<dyn GetHeight>) -> Self {
        Self {
            view_distance: 500.0,
            max_lod: 4,
            tile_size: 240,
            spawn_if_moved_by: 256.0,
            force_spawn: true,
            heightmap,
            max_height: 500.0,
            texture: Id::default(),
            unit_size: 1.0,
            mesh_size: MeshSize::Standard,
        }
    }

    /// Forces terrain to respown tiles
    pub fn respawn(&mut self) {
        self.force_spawn = true;
    }

    /// Returns downcasted reference to the height map
    pub fn heightmap<T: 'static>(&self) -> Option<&T> {
        self.heightmap.downcast_ref::<T>()
    }

    /// Returns downcasted mutable reference to the height map
    pub fn heightmap_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.heightmap.downcast_mut::<T>()
    }

    /// Generates terrain mesh
    pub fn polygonize(
        &self,
        position: VecXZ<f32>,
        offset: VecXZ<u32>,
        scale: u32
    ) -> Mesh {
        let vertices_per_side = self.mesh_size.vertices_per_side();
        let units_per_side = vertices_per_side - 1;

        let capacity = vertices_per_side * vertices_per_side;
        let mut positions = Vec::with_capacity(capacity);
        let mut uvs = Vec::with_capacity(capacity);
        let mut normals = vec![[0.0, 0.0, 0.0]; capacity];
        let mut indices = Vec::with_capacity(3 * 2 * units_per_side * units_per_side);

        let unit_size = self.unit_size * scale as f32;

        let mut zf = position.z;
        for z in 0..vertices_per_side {
            let mut xf = position.x;
            let map_z = offset.z + z;
            for x in 0..vertices_per_side {
                let map_x = offset.x + x;
                let height = self.heightmap.value(map_x as usize, map_z as usize);
                let world_y = self.max_height * height;

                positions.push([x as f32, y, world_z as f32]);
                uvs.push([
                    (x + offset) as f32 / 2.0 / offset as f32,
                    (z + offset) as f32 / 2.0 / offset as f32,
                ]);
                xf += unit_size;
            }
            zf += unit_size;
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

    /// Returns height in defined point
    pub fn height(&self, x: f32, z: f32) -> f32 {
        let size = self.heightmap.size() as f32;
        let offset = size as f32 / 2.0;
        let x = x + offset;
        let z = z + offset;
        if x < 0.0 || x >= size || z < 0.0 || z >= size {
            return 0.0;
        }
        self.max_height * self.heightmap.value(x.round() as usize, z.round() as usize)
    }

    /// Returns coordinate of intersection with mouse ray if one occurs
    pub fn ray_intersection(&self, ray: &Ray) -> Option<Vec3> {
        const RAY_RANGE: f32 = 4000.0;
        if let Some(direction) = ray.direction.as_ref() {
            if let Some(origin) = ray.origin.as_ref() {
                let target = origin + direction * RAY_RANGE;

                if self.is_under(&target) && !self.is_under(&origin) {
                    return Some(
                        self.binary_search_intersection(0, 0.0, RAY_RANGE, ray)
                    );
                }
            }
        }

        None
    }

    /// Returns true if point is under the terrain
    pub fn is_under(&self, point: &Vec3) -> bool {
        self.height(point.x, point.z) > point.y
    }

    fn binary_search_intersection(
        &self,
        depth: u32,
        min: f32,
        max: f32,
        ray: &Ray,
    ) -> Vec3 {
        const RECURSION_LIMIT: u32 = 200;
        const PRECISION: f32 = 0.01;
        let half = min + (max - min) / 2.0;

        let point = ray.point(half);
        let y = self.height(point.x, point.z);

        if depth == RECURSION_LIMIT || (y - point.y).abs() < PRECISION {
            return point;
        }

        let (min, max) = if self.is_under(&point) {
            (min, half)
        } else {
            (half, max)
        };

        self.binary_search_intersection(depth + 1, min, max, ray)
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Self::new(Box::new(Generator::default()))
    }
}
