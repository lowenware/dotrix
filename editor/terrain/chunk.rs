use dotrix::{
    assets::{Id, Mesh},
    components::{ Model },
    math::{Vec3, Vec3i},
    renderer::{ Transform },
    services::{ Assets, Camera, World },
    terrain::{
        Density,
        MarchingCubes,
    }
};

use crate::terrain::VoxelMap;

const CHUNK_SIZE: usize = 16;

pub struct Tile {
    pub position: Vec3i,
    pub ring: usize,
}

pub struct Chunk {
    pub position: Vec3i,
    pub ring: usize,
    pub size: usize,
    pub mesh: Option<Id<Mesh>>,
    pub changed: bool,
    pub hollow: bool,
    pub disabled: bool,
}

impl Chunk {
    pub fn new(position: Vec3i, ring: usize, size: usize) -> Self {
        Self {
            position,
            ring,
            size,
            mesh: None,
            hollow: false,
            changed: true,
            disabled: true,
        }
    }

    pub fn polygonize(
        &mut self,
        assets: &mut Assets,
        world: &mut World,
        voxel_map: &VoxelMap
    ) {
        let mc = MarchingCubes {
            size: Self::size() as usize,
            height: Self::size() as usize,
            ..Default::default()
        };
        let scale = (self.size / Chunk::size()) as f32;

        let (positions, _) = mc.polygonize(|x, y, z| voxel_map[x][y][z]);

        let len = positions.len();

        if len == 0 {
            self.hollow = true;
            return;
        }
        let uvs = Some(match self.ring {
            2 => vec![[0.0, 0.0]; len],
            3 => vec![[1.0, 0.0]; len],
            4 => vec![[1.0, 1.0]; len],
            _ => vec![[0.0, 1.0]; len],
        });

        if let Some(mesh_id) = self.mesh {
            let mesh = assets.get_mut(mesh_id).unwrap();
            mesh.positions = positions;
            mesh.uvs = uvs;
            mesh.normals.take();
            mesh.calculate();
            mesh.unload();

        } else {
            let mut mesh = Mesh {
                positions,
                uvs,
                ..Default::default()
            };
            mesh.calculate();

            let mesh = assets.store(mesh);

            let texture = assets.register("terrain");
            let half_size = (self.size / 2) as f32;

            let transform = Transform {
                translate: Vec3::new(
                    self.position.x as f32 - half_size,
                    self.position.y as f32 - half_size,
                    self.position.z as f32 - half_size,
                ),
                scale: Vec3::new(scale as f32, scale as f32, scale as f32),
                ..Default::default()
            };
            let tile = self.tile();

            world.spawn(
                Some((Model { mesh, texture, transform, ..Default::default() }, tile,))
            );

            self.mesh = Some(mesh);
        }
        self.changed = false;
    }

    pub fn tile(&self) -> Tile {
        Tile {
            position: self.position,
            ring: self.ring,
        }
    }

    pub fn size() -> usize {
        CHUNK_SIZE
    }
}
