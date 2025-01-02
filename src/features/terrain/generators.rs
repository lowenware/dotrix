use super::{ColorMap, HeightMap, LoD, Moisture, Terrain};
use crate::math::Vec3;
use crate::{Color, Mesh, VertexNormal, VertexPosition};

pub struct TileSetup<'a> {
    pub lod: LoD,
    pub map_offset_x: i32,
    pub map_offset_z: i32,
    pub position_x: i32,
    pub position_z: i32,
    pub heightmap: &'a HeightMap,
    pub moisturemap: &'a HeightMap,
    pub colormap: &'a ColorMap,
}

pub trait Generator: Send + Sync {
    fn tile_size(&self) -> u32;
    fn amplify_height(&self, heightmap_value: f32) -> f32;
    fn vertices_count(&self, lod: LoD) -> u32;
    fn indices_count(&self, lod: LoD) -> u32;
    fn indices(&self, lod: LoD) -> Vec<u32>;
    fn generate(&self, setup: &TileSetup) -> Terrain;
}

pub struct SimpleTerrain {
    pub tile_size: u32,
    pub height_factor: f32,
}

impl SimpleTerrain {
    pub fn new(tile_size: u32, height_factor: f32) -> Self {
        Self {
            tile_size,
            height_factor,
        }
    }
}

impl Generator for SimpleTerrain {
    fn tile_size(&self) -> u32 {
        self.tile_size
    }
    fn amplify_height(&self, heightmap_value: f32) -> f32 {
        self.height_factor * heightmap_value.powi(4)
    }
    fn vertices_count(&self, lod: LoD) -> u32 {
        let tile_size = self.tile_size / lod.factor();
        let vertices_per_side = tile_size + 1;
        vertices_per_side * vertices_per_side
    }
    fn indices_count(&self, lod: LoD) -> u32 {
        let tile_size = self.tile_size / lod.factor();
        tile_size * tile_size * 2 * 3
    }
    fn indices(&self, lod: LoD) -> Vec<u32> {
        let indices_count = self.indices_count(lod);
        let tile_size = self.tile_size / lod.factor();

        let mut indices = Vec::with_capacity(indices_count as usize);
        let vertices_per_side = tile_size + 1;
        for xi in 0..tile_size {
            let offset = xi * vertices_per_side;
            for zi in 0..tile_size {
                /*
                  A *---* B
                    | \ |
                  D *---* C
                */
                let index_a = offset + zi;
                let index_b = index_a + 1;
                let index_c = index_b + vertices_per_side;
                let index_d = index_a + vertices_per_side;

                indices.extend([
                    index_a, index_c, index_b, // face ACB
                    index_c, index_a, index_d, // face CAD
                ]);
            }
        }

        // log::debug!("    terrain[{}]: indices=\n{:?}", tile_size, indices);
        indices
    }
    fn generate(&self, setup: &TileSetup) -> Terrain {
        let lod_factor = setup.lod.factor();
        let sqares_per_side = self.tile_size / lod_factor;
        let vertices_per_side = sqares_per_side + 1;
        let vertices_count = vertices_per_side * vertices_per_side;
        let mut vertices = Vec::with_capacity(vertices_count as usize);
        let mut colors = Vec::with_capacity(vertices_count as usize);
        let mut normals = vec![[0.0, 0.0, 0.0]; vertices_count as usize];
        let mut moistures = vec![0.0; vertices_count as usize];

        // log::debug!(
        //     "Generate terrain: lod={}, vertices_count={}",
        //     self.lod.value(),
        //     vertices_count
        // );

        let map_offset = (setup.heightmap.size() as i32) / 2;

        for zi in 0..vertices_per_side {
            for xi in 0..vertices_per_side {
                let world_x = setup.position_x + (xi * lod_factor) as i32;
                let world_z = setup.position_z + (zi * lod_factor) as i32;
                let map_x = (map_offset + world_x).clamp(0, setup.heightmap.size() as i32) as u32;
                let map_z = (map_offset + world_z).clamp(0, setup.heightmap.size() as i32) as u32;
                let height = setup.heightmap.value(map_x, map_z);
                let world_y = self.amplify_height(height);

                /*
                    log::debug!(
                    "world:({};{}), map:({}:{}; size:{}, offset: {}), height:({}->{})",
                    world_x,
                    world_z,
                    map_x,
                    map_z,
                    heightmap.size(),
                    map_offset,
                    height,
                    world_y
                );*/

                vertices.push([world_x as f32, world_y, world_z as f32]);

                let moisture = setup.moisturemap.value(map_x, map_z);
                moistures.push(moisture);

                let color: Color<f32> =
                    (&Color::<u8>::from(setup.colormap.color(height, moisture))).into();
                /*
                let color = (color[0] as u32) << 24
                    | (color[1] as u32) << 16
                    | (color[2] as u32) << 8
                    | 255;
                    */
                // let color = match self.lod.value() {
                //     0 => Color::rgb(1.0, 0.0, 0.0),
                //     1 => Color::rgb(0.0, 1.0, 0.0),
                //     2 => Color::rgb(0.0, 0.0, 1.0),
                //     _ => Color::rgb(1.0, 1.0, 1.0),
                // };
                colors.push((&color).into());
            }
        }

        for zi in 0..sqares_per_side {
            let offset = zi * vertices_per_side;
            for xi in 0..sqares_per_side {
                /*
                  A *---* B
                    | \ |
                  D *---* C
                */
                let index_a = (offset + xi) as usize;
                let index_b = index_a + 1;
                let index_c = index_b + vertices_per_side as usize;
                let index_d = index_a + vertices_per_side as usize;

                let vertex_a = Vec3::from(vertices[index_a]);
                let vertex_b = Vec3::from(vertices[index_b]);
                let vertex_c = Vec3::from(vertices[index_c]);
                let vertex_d = Vec3::from(vertices[index_d]);

                // face ACB
                normals[index_b] = (vertex_b - vertex_a)
                    .cross(vertex_c - vertex_a)
                    .normalize()
                    .into();
                // face CAD
                normals[index_d] = (vertex_d - vertex_c)
                    .cross(vertex_a - vertex_c)
                    .normalize()
                    .into();
            }
        }

        // log::debug!("    terrain: lod={}", self.lod.value(),);
        // log::debug!("    terrain: vertices=\n{:?}", vertices);
        // log::debug!("    terrain: colors=\n{:?}", colors,);

        let mut mesh = Mesh::new("terrain");
        mesh.set_vertices::<VertexPosition>(vertices);
        mesh.set_vertices::<VertexNormal>(normals);
        mesh.set_vertices::<Color<f32>>(colors);
        mesh.set_vertices::<Moisture>(moistures);

        Terrain {
            mesh,
            lod: setup.lod,
            x: setup.position_x as f32,
            z: setup.position_z as f32,
        }
    }
}

pub struct LowPolyTerrain {
    pub tile_size: u32,
    pub height_factor: f32,
}

impl LowPolyTerrain {
    pub fn new(tile_size: u32, height_factor: f32) -> Self {
        Self {
            tile_size,
            height_factor,
        }
    }

    fn calculate_duplicated_vertex_index(index: u32, vertices_per_side: u32, row: u32) -> u32 {
        vertices_per_side * vertices_per_side + (index - vertices_per_side - (row - 1) * 2 - 1)
    }
    fn generate_sqaure_indices(square_x: u32, square_z: u32, vertices_per_side: u32) -> [u32; 6] {
        let offset = square_z * vertices_per_side;
        let top_left = offset + square_x;
        let top_right = top_left + 1;
        let bottom_left = top_left + vertices_per_side;
        let bottom_right = top_right + vertices_per_side;

        let last_square = vertices_per_side - 2;

        if square_x % 2 == 0 {
            // duplicate vertices
            let real_top_left = if square_x == 0 || square_z == 0 {
                top_left
            } else {
                Self::calculate_duplicated_vertex_index(top_left, vertices_per_side, square_z)
            };
            let real_top_right = if square_x == last_square || square_z == 0 {
                top_right
            } else {
                Self::calculate_duplicated_vertex_index(top_right, vertices_per_side, square_z)
            };
            if square_z % 2 == 0 {
                [
                    real_top_left,
                    bottom_left,
                    bottom_right,
                    real_top_right,
                    top_left,
                    bottom_right,
                ]
            } else {
                [
                    real_top_left,
                    bottom_left,
                    top_right,
                    real_top_right,
                    bottom_left,
                    bottom_right,
                ]
            }
        } else if square_z % 2 == 0 {
            [
                bottom_left,
                top_right,
                top_left,
                bottom_right,
                top_right,
                bottom_left,
            ]
        } else {
            [
                bottom_left,
                bottom_right,
                top_left,
                bottom_right,
                top_right,
                top_left,
            ]
        }
    }
}

impl Generator for LowPolyTerrain {
    fn tile_size(&self) -> u32 {
        self.tile_size
    }
    fn amplify_height(&self, heightmap_value: f32) -> f32 {
        self.height_factor * heightmap_value.powi(4)
    }
    fn vertices_count(&self, lod: LoD) -> u32 {
        let tile_size = self.tile_size / lod.factor();
        let vertices_per_side = tile_size + 1;
        let duplicated_per_side = tile_size - 1;
        vertices_per_side * vertices_per_side + duplicated_per_side * duplicated_per_side
    }
    fn indices_count(&self, lod: LoD) -> u32 {
        let tile_size = self.tile_size / lod.factor();
        tile_size * tile_size * 2 * 3
    }
    fn indices(&self, lod: LoD) -> Vec<u32> {
        let indices_count = self.indices_count(lod);
        let tile_size = self.tile_size / lod.factor();
        let squares_per_side = tile_size;
        let vertices_per_side = tile_size + 1;
        let mut indices = Vec::with_capacity(indices_count as usize);

        for square_z in 0..squares_per_side {
            for square_x in 0..squares_per_side {
                let square_indices =
                    Self::generate_sqaure_indices(square_x, square_z, vertices_per_side);
                indices.extend(square_indices.into_iter());
            }
        }
        // log::debug!("    terrain[{}]: indices=\n{:?}", tile_size, indices);
        indices
    }
    fn generate(&self, setup: &TileSetup) -> Terrain {
        let lod_factor = setup.lod.factor();
        let squares_per_side = self.tile_size / lod_factor;
        let vertices_per_side = squares_per_side + 1;
        let duplicates_per_side = squares_per_side - 1;
        let unique_vertices_count = vertices_per_side * vertices_per_side;
        let vertices_count = unique_vertices_count + duplicates_per_side * duplicates_per_side;
        let mut vertices = vec![[0.0, 0.0, 0.0]; vertices_count as usize];
        let mut colors = vec![[0.0, 0.0, 0.0, 0.0]; vertices_count as usize];
        let mut normals = vec![[0.0, 0.0, 0.0]; vertices_count as usize];
        let mut moistures = vec![0.0; vertices_count as usize];

        // log::debug!(
        //     "Generate terrain: lod={}, vertices_count={}",
        //     self.lod.value(),
        //     vertices_count
        // );

        let map_offset = (setup.heightmap.size() as i32) / 2;
        let mut vertex_cursor = 0;
        let mut duplicates_cursor = unique_vertices_count as usize;

        for zi in 0..vertices_per_side {
            for xi in 0..vertices_per_side {
                let world_x = setup.position_x + (xi * lod_factor) as i32;
                let world_z = setup.position_z + (zi * lod_factor) as i32;
                let map_x = (map_offset + world_x).clamp(0, setup.heightmap.size() as i32) as u32;
                let map_z = (map_offset + world_z).clamp(0, setup.heightmap.size() as i32) as u32;
                let height = setup.heightmap.value(map_x, map_z);
                let world_y = self.amplify_height(height);

                /*
                    log::debug!(
                    "world:({};{}), map:({}:{}; size:{}, offset: {}), height:({}->{})",
                    world_x,
                    world_z,
                    map_x,
                    map_z,
                    heightmap.size(),
                    map_offset,
                    height,
                    world_y
                );*/

                vertices[vertex_cursor] = [world_x as f32, world_y, world_z as f32];
                let moisture = setup.moisturemap.value(map_x, map_z);
                let color: Color<f32> =
                    (&Color::<u8>::from(setup.colormap.color(height, moisture))).into();
                // let color: Color<f32> = Color::grey();
                colors[vertex_cursor] = (&color).into();
                moistures[vertex_cursor] = moisture;
                if xi != 0 && xi != squares_per_side && zi != 0 && zi != squares_per_side {
                    // duplicate vertex
                    vertices[duplicates_cursor] = [world_x as f32, world_y, world_z as f32];
                    colors[duplicates_cursor] = (&color).into();
                    duplicates_cursor += 1;
                }
                vertex_cursor += 1;
            }
        }

        for square_z in 0..squares_per_side {
            for square_x in 0..squares_per_side {
                let indices = Self::generate_sqaure_indices(square_x, square_z, vertices_per_side);

                // log::debug!("square: ({}, {}) -> {:?}", square_x, square_z, indices));

                for triangle in 0..2 {
                    let base_index = triangle * 3;
                    let index0 = indices[base_index] as usize;
                    let index1 = indices[base_index + 1] as usize;
                    let index2 = indices[base_index + 2] as usize;
                    normals[index0] = (Vec3::from(vertices[index1]) - Vec3::from(vertices[index0]))
                        .cross(Vec3::from(vertices[index2]) - Vec3::from(vertices[index0]))
                        .normalize()
                        .into();
                }
            }
        }

        // log::debug!("    terrain: lod={}", self.lod.value(),);
        // log::debug!("    terrain: vertices=\n{:?}", vertices);
        // log::debug!("    terrain: colors=\n{:?}", colors,);

        let mut mesh = Mesh::new("terrain");
        mesh.set_vertices::<VertexPosition>(vertices);
        mesh.set_vertices::<VertexNormal>(normals);
        mesh.set_vertices::<Color<f32>>(colors);
        mesh.set_vertices::<Moisture>(moistures);

        Terrain {
            mesh,
            lod: setup.lod,
            x: setup.position_x as f32,
            z: setup.position_z as f32,
        }
    }
}
