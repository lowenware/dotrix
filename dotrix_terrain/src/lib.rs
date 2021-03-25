//! Dotrix terrain implementation
//!
//! This crate is under active development
#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

pub const CHUNK_SIZE: usize = 16;

mod density;
mod grid;
mod marching_cubes;
mod voxel_map;
pub mod octree;

pub use density::*;
pub use grid::*;
pub use marching_cubes::*;
pub use voxel_map::*;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{ Duration, Instant },
};
use noise::{ NoiseFn, Fbm };
use rayon::prelude::*;

use dotrix_core::{
    assets::{ Id, Mesh },
    components::{ Model, WireFrame },
    ecs::{ Const, Mut, Context },
    renderer::{ Transform },
    services::{ Assets, Camera, World },
};

use dotrix_math::{ Point3, Vec3, Vec3i, MetricSpace };

use octree::{Octree, Node as OctreeNode};

/// Level of details identified by number
pub struct Lod(pub usize);

/// Number of density values per side of the block inside of the grid
pub const GRID_BLOCK_SIZE: usize = CHUNK_SIZE;

// TODO: unify with octree
const LEFT_TOP_BACK: u8 = 0;
const RIGHT_TOP_BACK: u8 = 1;
const RIGHT_TOP_FRONT: u8 = 2;
const LEFT_TOP_FRONT: u8 = 3;
const LEFT_BOTTOM_BACK: u8 = 4;
const RIGHT_BOTTOM_BACK: u8 = 5;
const RIGHT_BOTTOM_FRONT: u8 = 6;
const LEFT_BOTTOM_FRONT: u8 = 7;

impl Lod {
    /// Get scale of the terrain chunk for specified [`Lod`]
    pub fn scale(&self) -> i32 {
        (2_i32).pow(self.0 as u32)
    }
}

/// Service for terrain management
pub struct Terrain {
    /// Last position of the viewer, the terrain is being around that point
    pub last_viewer_position: Option<Point3>,
    /// Terrain will be regenerated if viewer moves by this threshold distance
    pub update_if_moved_by: f32,
    /// Square of the view distance
    pub view_distance2: f32,
    /// Voxel mapping
    pub octree: Octree,
    /// Changes tracking flag, if `true` terrain will be regenerated
    pub changed: bool,
    /// Highest [`Lod`] limitation
    pub lod: usize,
    /// Generation time tracking
    pub generated_in: Duration,
    /// Density grid
    pub grid: Grid,
}

impl Terrain {
    /// Constructs new service instance
    pub fn new() -> Self {
        let map_size = 16;
        let grid_size = 32;
        let update_if_moved_by = map_size as f32 * 0.5;
        let mut octree = Octree::new(Vec3i::new(0, 0, 0), grid_size);
        octree.store(Vec3i::new(grid_size as i32 / 4, grid_size as i32 / 4, grid_size as i32 / 4));
        Self {
            update_if_moved_by: update_if_moved_by * update_if_moved_by,
            last_viewer_position: None,
            view_distance2: 768.0 * 768.0 + 768.0 * 768.0 + 768.0 * 768.0,
            octree,
            changed: false,
            lod: 3,
            generated_in: Duration::from_secs(0),
            grid: Grid::flat(grid_size + 1, GRID_BLOCK_SIZE),
        }
    }

    /// Populates the voxel map with noise
    pub fn populate(&mut self, noise: &Fbm, amplitude: f64, scale: f64) {
        return;
    }

    fn spawn(&mut self, target: Point3, instances: &mut HashMap<Vec3i, Instance>) {
        let instances = Arc::new(Mutex::new(instances));
        self.spawn_node(target, instances, &self.octree.root, 0xFF, true);
    }

    fn spawn_node(
        &self,
        target: Point3,
        instances: Arc<Mutex<&mut HashMap<Vec3i, Instance>>>,
        node_key: &Vec3i,
        index: u8,
        recursive: bool,
    ) {
        if let Some(node) = self.octree.load(&node_key) {
            // let lod = Lod(node.level);

            if !recursive || node.level == self.lod || node.children.is_none(){
                // Get stored instance or make new from the node
                let mut instances = instances.lock().unwrap();
                if let Some(instance) = instances.get_mut(node_key) {
                    // check if has LOD round up has changed
                    instance.disabled = false;
                } else {
                    instances.insert(*node_key, Instance::from(*node_key, &node, index));
                }
            } else {
                let min_view_distance2 = 0.75 * node.size as f32 * 0.75 * node.size as f32;
                let children = node.children.as_ref().unwrap();
                // Calculate node configuration: what nodes has to be rendered with higher LOD
                let mut configuration: u8 = 0;
                let mut in_view_distance: u8 = 0;
                for (i, child) in children.iter().enumerate() {
                    let point = Point3::new(child.x as f32, child.y as f32, child.z as f32);
                    let distance2 = target.distance2(point);
                    if distance2 < self.view_distance2 {
                        let bit = 1 << i;
                        if distance2 <= min_view_distance2 {
                            configuration |= bit;
                        }
                        in_view_distance |= bit;
                    }
                }
                children.into_par_iter().enumerate().for_each(|(i, child)| {
                    // Calculate LOD round up
                    let bit = 1 << i;
                    if in_view_distance & bit == 0 { return; }
                    self.spawn_node(
                        target,
                        Arc::clone(&instances),
                        child,
                        i as u8,
                        configuration & bit != 0
                    );
                });
            }
        }
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns default [`Terrain`] service
#[inline(always)]
pub fn service() -> Terrain {
    Terrain::default()
}

/// [`Terrain`] block component
#[derive(Debug)]
pub struct Block {
    /// Block position (center of the cube)
    pub position: Vec3i,
    /// Position of the block's front bottom left corer
    pub bound_min: Vec3i,
    /// Position of the block's back top bottom right corer
    pub bound_max: Vec3i,
    /// size of the voxel (not block)
    pub voxel_size: usize,
}

/// Chunk instance of the terrain
pub struct Instance {
    position: Vec3i,
    /// Postition index in parent cube
    index: u8,
    level: u8,
    mesh: Option<Id<Mesh>>,
    size: usize,
    empty: bool,
    disabled: bool,
    updated: bool,
    round_up: [u8; 3],
}

impl Instance {
    /// Constructs [`Instance`] from [`OctreeNode`] at some position
    pub fn from(position: Vec3i, node: &OctreeNode, index: u8) -> Self {
        Self {
            position,
            index,
            level: node.level as u8,
            size: node.size,
            mesh: None,
            disabled: false,
            empty: false,
            updated: true,
            round_up: [0; 3],
        }
    }

    fn resolve_seams(
        &self,
        density: &mut Vec<Vec<Vec<f32>>>,
        round_up: &[u8; 3],
    ) {
        let (x, y, z) = match self.index {
            LEFT_TOP_BACK => (0, 16, 0),
            RIGHT_TOP_BACK => (16, 16, 0),
            RIGHT_TOP_FRONT => (16, 16, 16),
            LEFT_TOP_FRONT => (0, 16, 16),
            LEFT_BOTTOM_BACK => (0, 0, 0),
            RIGHT_BOTTOM_BACK => (16, 0, 0),
            RIGHT_BOTTOM_FRONT => (16, 0, 16),
            LEFT_BOTTOM_FRONT => (0, 0, 16),
            _ => panic!("Cube has only 8 sides")
        };

        if round_up[0] > 0 {
            Self::resolve_seams_x(density, (self.level - round_up[0]) as usize + 1, x);
        }
        if round_up[1] > 0 {
            Self::resolve_seams_y(density, (self.level - round_up[1]) as usize + 1, y);
        }
        if round_up[2] > 0 {
            Self::resolve_seams_z(density, (self.level - round_up[2]) as usize + 1, z);

        }
    }


    /* This algorithm is more effective, but for some reason brings cracks and floating stones
     * where current one does not. Could happen, that it is connected to write / read into memory
     * and f32 precision. Lets keep it here, until we settle all the things down.
     *
    fn resolve_seams_x(map: &mut VoxelMap, step: usize, x: usize) {
        for y in (0..16).step_by(step) {
            for z in (0..16).step_by(step) {
                let mut v = map[x][y][z];
                let s = (map[x][y][z + step] - v) / step as f32;
                for zi in 1..step {
                    v += s;
                    map[x][y][z + zi] = v;
                    if y != 0 {
                        let y0 = y - step;
                        let mut v0 = map[x][y0][z + zi];
                        let s0 = (v - v0) / step as f32;
                        for yi in 1..step {
                            v0 += s0;
                            map[x][y0 + yi][z + zi] = v0;
                        }
                    }
                }
            }
        }
    }
    */
    fn resolve_seams_x(density: &mut Vec<Vec<Vec<f32>>>, step: usize, x: usize) {
        for y in (0..16).step_by(step) {
            for z in (0..16).step_by(step) {
                let mut v0 = density[x][y][z];
                let s0 = (density[x][y][z + step] - v0) / step as f32;
                let mut v1 = density[x][y + step][z];
                let s1 = (density[x][y + step][z + step] - v1) / step as f32;

                for zi in 1..step {
                    v0 += s0;
                    v1 += s1;
                    density[x][y][z + zi] = v0;
                    density[x][y + step][z + zi] = v1;
                    let mut v = v0;
                    let s = (v1 - v0) / step as f32;
                    for yi in 1..step {
                        v += s;
                        density[x][y + yi][z + zi] = v;
                    }
                }
            }
        }
    }

    fn resolve_seams_y(density: &mut Vec<Vec<Vec<f32>>>, step: usize, y: usize) {
        for x in (0..17).step_by(step) {
            for z in (0..16).step_by(step) {
                let mut v = density[x][y][z];
                let v1 = density[x][y][z + step];
                let s = (v1 - v) / step as f32;
                for zi in 1..step {
                    let old = density[x][y][z + zi];
                    if (v1 < 0.0 && v < 0.0 && old > 0.0) || (v1 > 0.0 && v > 0.0 && old < 0.0) {
                        v = 0.0;
                    } else {
                        v += s;
                    }
                    density[x][y][z + zi] = v;
                }
            }
        }

        for z in (0..17).step_by(step) {
            for x in (0..16).step_by(step) {
                let mut v = density[x][y][z];
                let v1 = density[x + step][y][z];
                let s = (v1 - v) / step as f32;
                for xi in 1..step {
                    let old = density[x + xi][y][z];
                    if (v1 < 0.0 && v < 0.0 && old > 0.0) || (v1 > 0.0 && v > 0.0 && old < 0.0) {
                        v = 0.0;
                    } else {
                        v += s;
                    }
                    density[x + xi][y][z] = v;
                }
            }
        }

        for x in (0..16).step_by(step) {
            for z in (0..16).step_by(step) {
                let v1 = (density[x + step][y][z + step] - density[x][y][z]) / step as f32 + density[x][y][z];
                let v2 = (density[x][y][z + step] - density[x + step][y][z]) / step as f32 + density[x + step][y][z];
                density[x + 1][y][z + 1] = if v1 < v2 { v1 } else { v2 };
            }
        }
    }

    fn resolve_seams_z(density: &mut Vec<Vec<Vec<f32>>>, step: usize, z: usize) {
        for x in (0..16).step_by(step) {
            for y in (0..16).step_by(step) {
                let mut v0 = density[x][y][z];
                let s0 = (density[x + step][y][z] - v0) / step as f32;
                let mut v1 = density[x][y + step][z];
                let s1 = (density[x + step][y + step][z] - v1) / step as f32;
                for xi in 1..step {
                    v0 += s0;
                    v1 += s1;
                    density[x + xi][y][z] = v0;
                    density[x + xi][y + step][z] = v1;
                    let mut v = v0;
                    let s = (v1 - v0) / step as f32;
                    for yi in 1..step {
                        v += s;
                        density[x + xi][y + yi][z] = v;
                    }
                }
            }
        }
    }

    /// Generates polygons of the [`Instance`]
    pub fn polygonize(
        &mut self,
        assets: &mut Assets,
        world: &mut World,
        mut density: Density,
        round_up: &[u8; 3],
    ) {

        let map_size = 16;
        let mc = MarchingCubes {
            size: map_size,
            ..Default::default()
        };
        let scale = (self.size / map_size) as f32;

        // self.resolve_seams(&mut density, round_up);
        // TODO: repair seams patching
        let map_size_sq = (map_size + 1) * (map_size + 1);
        let density_values = density.values().expect("Density values to be set");
        let (positions, _) = mc.polygonize(
            |x, y, z| density_values[map_size_sq * x + (map_size + 1) * y + z] as f32
        );

        let len = positions.len();

        if len == 0 {
            self.empty = true;
            return;
        }
        self.round_up = *round_up;
        let uvs = Some(vec![[1.0, 0.0]; len]);
        /* match self.ring {
            2 => vec![[0.0, 0.0]; len],
            3 => vec![[1.0, 0.0]; len],
            4 => vec![[1.0, 1.0]; len],
            _ => vec![[0.0, 1.0]; len],
        }); */

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
            let block = self.block();
            let wires = assets.find("wires_gray").expect("wires_gray to be loaded");
            let wires_transform = Transform {
                translate: Vec3::new(
                    self.position.x as f32,
                    self.position.y as f32,
                    self.position.z as f32,
                ),
                scale: Vec3::new(half_size, half_size, half_size),
                ..Default::default()
            };

            world.spawn(
                Some((
                    Model { mesh, texture, transform, ..Default::default() },
                    WireFrame { wires, transform: wires_transform, ..Default::default() },
                    block
                ))
            );

            self.mesh = Some(mesh);
        }
        self.updated = false;
    }

    /// Constructs [`Block`] component from the [`Instance`]
    pub fn block(&self) -> Block {
        let half_size = self.size as i32 / 2;
        Block {
            position: self.position,
            bound_min: Vec3i::new(
                self.position.x - half_size,
                self.position.y - half_size,
                self.position.z - half_size
            ),
            bound_max: Vec3i::new(
                self.position.x + half_size,
                self.position.y + half_size,
                self.position.z + half_size
            ),
            voxel_size: self.size / 16,
        }
    }

    fn parent(position: &Vec3i, size: i32, index: u8) -> Vec3i {
        let half_size = size / 2;
        position + Vec3i::from(match index {
            LEFT_TOP_BACK => [half_size, -half_size, half_size],
            RIGHT_TOP_BACK => [-half_size, -half_size, half_size],
            RIGHT_TOP_FRONT => [-half_size, -half_size, -half_size],
            LEFT_TOP_FRONT => [half_size, -half_size, -half_size],
            LEFT_BOTTOM_BACK => [half_size, half_size, half_size],
            RIGHT_BOTTOM_BACK => [-half_size, half_size, half_size],
            RIGHT_BOTTOM_FRONT => [-half_size, half_size, -half_size],
            LEFT_BOTTOM_FRONT => [half_size, half_size, -half_size],
            _ => panic!("Cube has only 8 sides")
        })
    }

    /// Finds what Instances surrounds the current one
    pub fn round_up(&self, instances: &HashMap<Vec3i, Instance>) -> [u8; 3] {
        let size = self.size as i32;
        let parent_size = 2 * size;
        let parent = Self::parent(&self.position, size, self.index);
        let mut result = [0; 3];
        // Get 3 neighbours from outside cubes
        let round_up = match self.index {
            LEFT_TOP_BACK => [
                Vec3i::new(parent.x - parent_size, parent.y, parent.z),
                Vec3i::new(parent.x, parent.y + parent_size, parent.z),
                Vec3i::new(parent.x, parent.y, parent.z - parent_size),
            ],
            RIGHT_TOP_BACK => [
                Vec3i::new(parent.x + parent_size, parent.y, parent.z),
                Vec3i::new(parent.x, parent.y + parent_size, parent.z),
                Vec3i::new(parent.x, parent.y, parent.z - parent_size),
            ],
            RIGHT_TOP_FRONT => [
                Vec3i::new(parent.x + parent_size, parent.y, parent.z),
                Vec3i::new(parent.x, parent.y + parent_size, parent.z),
                Vec3i::new(parent.x, parent.y, parent.z + parent_size),
            ],
            LEFT_TOP_FRONT => [
                Vec3i::new(parent.x - parent_size, parent.y, parent.z),
                Vec3i::new(parent.x, parent.y + parent_size, parent.z),
                Vec3i::new(parent.x, parent.y, parent.z + parent_size),
            ],
            LEFT_BOTTOM_BACK => [
                Vec3i::new(parent.x - parent_size, parent.y, parent.z),
                Vec3i::new(parent.x, parent.y - parent_size, parent.z),
                Vec3i::new(parent.x, parent.y, parent.z - parent_size),
            ],
            RIGHT_BOTTOM_BACK => [
                Vec3i::new(parent.x + parent_size, parent.y, parent.z),
                Vec3i::new(parent.x, parent.y - parent_size, parent.z),
                Vec3i::new(parent.x, parent.y, parent.z - parent_size),
            ],
            RIGHT_BOTTOM_FRONT => [
                Vec3i::new(parent.x + parent_size, parent.y, parent.z),
                Vec3i::new(parent.x, parent.y - parent_size, parent.z),
                Vec3i::new(parent.x, parent.y, parent.z + parent_size),
            ],
            LEFT_BOTTOM_FRONT => [
                Vec3i::new(parent.x - parent_size, parent.y, parent.z),
                Vec3i::new(parent.x, parent.y - parent_size, parent.z),
                Vec3i::new(parent.x, parent.y, parent.z + parent_size),
            ],
            _ => panic!("Cube has only 8 sides")
        };

        for i in 0..3 {
            result[i] = Self::recursive_level(&round_up[i], size, self.level, instances);
        }

        result
    }

    fn recursive_level(
        block: &Vec3i,
        size: i32,
        level: u8,
        instances: &HashMap<Vec3i, Instance>
    ) -> u8 {
        if let Some(instance) = instances.get(block) {
            if !instance.disabled {
                return instance.level;
            }
        }

        if level > 1 {
            let parent_size = size * 2;
            let parent = Vec3i::new(
                (block.x as f32 / parent_size as f32).floor() as i32 * parent_size + size,
                (block.y as f32 / parent_size as f32).floor() as i32 * parent_size + size,
                (block.z as f32 / parent_size as f32).floor() as i32 * parent_size + size,
            );
            return Self::recursive_level(&parent, parent_size, level - 1, instances);
        }

        0
    }

    fn is_same_round_up(first: &[u8; 3], second: &[u8; 3]) -> bool {
        first[0] == second[0] && first[1] == second[1] && first[2] == second[2]
    }
}

/// Terrain [`spawn`] system context
#[derive(Default)]
pub struct Spawner {
    /// Previously spawned instances
    pub instances: HashMap<Vec3i, Instance>,
}

/// System to spawn the terrain
pub fn spawn(
    mut ctx: Context<Spawner>,
    camera: Const<Camera>,
    mut terrain: Mut<Terrain>,
    mut assets: Mut<Assets>,
    mut world: Mut<World>,
) {
    let now = Instant::now();
    // let viewer = Point3::new(0.0, 0.0, 0.0);
    let viewer = camera.target;

    // check if update is necessary
    if let Some(last_viewer_position) = terrain.last_viewer_position.as_ref() {
        let dx = viewer.x - last_viewer_position.x;
        let dy = viewer.y - last_viewer_position.y;
        let dz = viewer.z - last_viewer_position.z;
        if dx * dx + dy * dy + dz * dz < terrain.update_if_moved_by && !terrain.changed {
            return;
        }
    }
    terrain.last_viewer_position = Some(viewer);

    if terrain.changed {
        ctx.instances.clear();
        // TODO: rework entities removing
        let query = world.query::<(&mut Model, &mut Block)>();
        for (model, block) in query {
            assets.remove(model.mesh);
            model.disabled = true;
            block.position.x = 0;
            block.position.y = 0;
            block.position.z = 0;
        }
        terrain.changed = false;
    }

    // disable all instances
    for instance in ctx.instances.values_mut() {
        instance.disabled = true;
    }

    terrain.spawn(viewer, &mut ctx.instances);

    let round_ups = ctx.instances.iter()
        .map(|(&key, instance)| {
            (key, if instance.empty {
                [0, 0, 0]
            } else {
                instance.round_up(&ctx.instances)
            })
        }).collect::<HashMap<_, _>>();

    for (key, instance) in ctx.instances.iter_mut() {
        if instance.empty {
            continue;
        }
        let round_up = round_ups.get(key).expect("Each instance must have a roundup");
        if instance.updated || !Instance::is_same_round_up(&instance.round_up, round_up) {
            // println!("polygonize: {:?}", instance.position);
            let half_size = instance.size as i32 / 2;
            let base = Vec3i::new(key.x - half_size, key.y - half_size, key.z - half_size);
            let density = terrain.grid.load(base, 17);
            // println!("{:?} -> {:?}\n", base, density);
            instance.polygonize(&mut assets, &mut world, density, round_up);
        }
    }

    let query = world.query::<(&mut Model, &mut WireFrame, &Block)>();
    for (model, wire_frame, block) in query {
        model.disabled = ctx.instances.get(&block.position)
            .map(|instance| instance.disabled || instance.mesh.is_none())
            .unwrap_or_else(|| {
                // println!("Not instanced {:?}", block.position);

                /*
                println!("Chunk not found: {:?}", index);
                let mesh = model.mesh;
                if !mesh.is_null() {
                    // assets.remove(mesh);
                }
                model.mesh = Id::new(0);
                unused_entities += 1;
                */
                true
            });
        wire_frame.disabled = model.disabled;
    }

    terrain.generated_in = now.elapsed();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_calculation_by_coordinate() {
        let parent = Vec3i::new(0, 0, 0);
        let size = 256;
        let left_top_back = Vec3i::new(parent.x - size, parent.y + size, parent.z - size);
        let right_top_back = Vec3i::new(parent.x + size, parent.y + size, parent.z - size);
        let right_top_front = Vec3i::new(parent.x + size, parent.y + size, parent.z + size);
        let left_top_front = Vec3i::new(parent.x - size, parent.y + size, parent.z + size);
        let left_bottom_back = Vec3i::new(parent.x - size, parent.y - size, parent.z - size);
        let right_bottom_back = Vec3i::new(parent.x + size, parent.y - size, parent.z - size);
        let right_bottom_front = Vec3i::new(parent.x + size, parent.y - size, parent.z + size);
        let left_bottom_front = Vec3i::new(parent.x - size, parent.y - size, parent.z + size);

        assert_eq!(Instance::parent(&left_top_back, 2 * size, 0), parent);
        assert_eq!(Instance::parent(&right_top_back, 2 * size, 1), parent);
        assert_eq!(Instance::parent(&right_top_front, 2 * size, 2), parent);
        assert_eq!(Instance::parent(&left_top_front, 2 * size, 3), parent);
        assert_eq!(Instance::parent(&left_bottom_back, 2 * size, 4), parent);
        assert_eq!(Instance::parent(&right_bottom_back, 2 * size, 5), parent);
        assert_eq!(Instance::parent(&right_bottom_front, 2 * size, 6), parent);
        assert_eq!(Instance::parent(&left_bottom_front, 2 * size, 7), parent);
    }

}
