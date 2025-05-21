mod colormap;
mod generators;
mod heightmap;
mod renderer;

use std::collections::HashMap;

use crate::{Any, Assets, Camera, Entity, Id, Mesh, Mut, Ref, Task, VertexAttribute, World};
pub use colormap::ColorMap;
pub use generators::{Generator, LowPolyTerrain, SimpleTerrain, TileSetup};
pub use heightmap::{FalloffConfig, HeightMap, NoiseConfig};
pub use renderer::{LodSetup, RenderTerrain, RenderTerrainSetup};

pub const TILE_MIN_SIZE: u32 = 2;
pub const TILE_MAX_SIZE: u32 = 254;
pub const DEFAULT_TILE_SIZE: u32 = 120;
pub const DEFAULT_TILES_IN_VIEW_RANGE: u32 = 4;
pub const DEFAULT_MAX_LODS: u32 = 3;
pub const DEFAULT_HEIGHT_AMPLIFIER: f32 = 100.0;

pub struct Moisture {
    pub value: f32,
}

impl VertexAttribute for Moisture {
    type Raw = f32;
    fn name() -> &'static str {
        "Moisture"
    }
    fn pack(&self) -> Self::Raw {
        self.value
    }
    fn format() -> crate::Format {
        crate::Format::Float32
    }
}

/// Terrain Level of Details
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct LoD {
    value: u32,
}

impl LoD {
    pub fn new(value: u32) -> Self {
        LoD { value }
    }
    pub fn factor(&self) -> u32 {
        if self.value != 0 {
            self.value * 2
        } else {
            1
        }
    }
    pub fn value(&self) -> u32 {
        self.value
    }
}

/// Terrain component
pub struct Terrain {
    /// Terrain mesh
    pub mesh: Mesh,
    /// Terrain X world position
    pub x: f32,
    /// Terrain Z world position
    pub z: f32,
    /// Terrain Level Of Details (0..)
    pub lod: LoD,
}

pub struct SpawnTerrain {
    /// number of sections (each section represents a LOD)
    tiles_in_view_range: u32,
    /// maximal number of LODs
    max_lods: u32,
    /// spawned entities index
    index: HashMap<Tile, Id<Entity>>,
    /// Height Map asset name
    heightmap: String,
    /// Moisture Map asset name
    moisturemap: String,
    /// Color Map asset name
    colormap: String,
    /// generator
    generator: Box<dyn Generator>,
}

impl Default for SpawnTerrain {
    fn default() -> Self {
        let total_visible_tiles = 4 * DEFAULT_TILES_IN_VIEW_RANGE * DEFAULT_TILES_IN_VIEW_RANGE;
        Self {
            tiles_in_view_range: DEFAULT_TILES_IN_VIEW_RANGE,
            max_lods: DEFAULT_MAX_LODS,
            index: HashMap::with_capacity(total_visible_tiles as usize),
            heightmap: String::from("terrain::heightmap"),
            moisturemap: String::from("terrain::moisturemap"),
            colormap: String::from("terrain::colormap"),
            generator: Box::new(SimpleTerrain::new(
                DEFAULT_TILE_SIZE,
                DEFAULT_HEIGHT_AMPLIFIER,
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Tile {
    xi: i32,
    zi: i32,
    lod: LoD,
}

impl SpawnTerrain {
    pub fn with_tiles_in_view_range(mut self, tiles_in_view_range: u32) -> Self {
        self.tiles_in_view_range = tiles_in_view_range;
        self
    }

    pub fn with_max_lods(mut self, max_lods: u32) -> Self {
        self.max_lods = max_lods;
        self
    }

    pub fn with_heightmap(mut self, heightmap: impl Into<String>) -> Self {
        self.heightmap = heightmap.into();
        self
    }

    pub fn with_moisturemap(mut self, moisturemap: impl Into<String>) -> Self {
        self.moisturemap = moisturemap.into();
        self
    }

    pub fn with_colormap(mut self, colormap: impl Into<String>) -> Self {
        self.colormap = colormap.into();
        self
    }

    pub fn with_generator(mut self, generator: Box<dyn Generator>) -> Self {
        self.generator = generator;
        self
    }

    fn get_tile_lod(&self, xi: i32, zi: i32) -> LoD {
        for i in 0..self.max_lods {
            if self.tiles_in_view_range < i {
                break;
            }

            let boundary = (self.tiles_in_view_range - i) as i32;
            if xi == -boundary || xi == boundary - 1 || zi == -boundary || zi == boundary - 1 {
                return LoD::new(self.max_lods - i - 1);
            }
        }
        LoD::default()
    }
}

#[derive(Default, Debug)]
pub struct SpawnTerrainOutput {
    pub tiles_to_exile: Vec<Id<Entity>>, // must have
    pub tiles_to_spawn: Vec<Id<Entity>>, // not very usefull
    pub scene: Vec<Id<Entity>>,          // could be usefull
}

impl Task for SpawnTerrain {
    type Output = SpawnTerrainOutput;
    type Context = (Any<Camera>, Mut<World>, Ref<Assets>);

    fn run(&mut self, (camera, mut world, assets): Self::Context) -> Self::Output {
        // log::debug!("SpawnTerrain::run()");
        let mut index = HashMap::with_capacity(self.index.capacity());
        let mut tiles_to_spawn: Vec<Id<Entity>> = Vec::with_capacity(4);
        let pos_x = camera.target.x;
        let pos_z = camera.target.z;
        //log::debug!("SpawnTerrain::run(@{};{})", pos_x, pos_z);
        let tile_size = self.generator.tile_size();

        let terrain_xi = (pos_x / (tile_size as f32)).floor() as i32;
        let terrain_zi = (pos_z / (tile_size as f32)).floor() as i32;

        let tiles_in_view_range = self.tiles_in_view_range as i32;

        let heightmap = match assets
            .find::<HeightMap>(&self.heightmap)
            .and_then(|id| assets.get(id))
        {
            Some(heightmap) => heightmap,
            None => {
                log::warn!("Terrain `{}` asset is not ready", self.heightmap);
                return SpawnTerrainOutput::default();
            }
        };

        let moisturemap = match assets
            .find::<HeightMap>(&self.moisturemap)
            .and_then(|id| assets.get(id))
        {
            Some(moisturemap) => moisturemap,
            None => {
                log::warn!("Terrain `{}` asset is not ready", self.moisturemap);
                return SpawnTerrainOutput::default();
            }
        };

        let colormap = match assets
            .find::<ColorMap>(&self.colormap)
            .and_then(|id| assets.get(id))
        {
            Some(colormap) => colormap,
            None => {
                log::warn!("Terrain `{}` asset is not ready", self.colormap);
                return SpawnTerrainOutput::default();
            }
        };

        for offset_z in -tiles_in_view_range..tiles_in_view_range {
            for offset_x in -tiles_in_view_range..tiles_in_view_range {
                let zi = terrain_zi + offset_z;
                let xi = terrain_xi + offset_x;
                let lod = self.get_tile_lod(offset_x, offset_z);
                // log::debug!("tile LOD: {};{} -> {}", xi, zi, lod.value());
                let tile = Tile { xi, zi, lod };

                if let Some(entity_id) = self.index.remove(&tile) {
                    index.insert(tile, entity_id);
                } else {
                    let map_offset = heightmap.size() as i32 / 2;
                    let setup = TileSetup {
                        lod,
                        position_x: xi * tile_size as i32,
                        position_z: zi * tile_size as i32,
                        map_offset_x: -map_offset,
                        map_offset_z: -map_offset,
                        heightmap,
                        moisturemap,
                        colormap,
                    };

                    let terrain = self.generator.generate(&setup);

                    let entity_id = world
                        .spawn(Some(Entity::empty().with(terrain)))
                        .next()
                        .expect("Terrain entity id was not returned after spawning");
                    index.insert(tile, entity_id);
                    log::debug!(
                        "ECS_TERRAIN: spawn {entity_id:?} ({xi};{zi} -> {})",
                        lod.value()
                    );
                    tiles_to_spawn.push(entity_id);
                }
            }
        }

        // for (tile, entity) in index.iter() {
        //     log::debug!("terrain setup: [{:?}]: {:?}", entity, tile);
        // }

        let mut tiles_to_exile = Vec::with_capacity(self.index.len());
        // Clear terrain out of view range
        for entity_id in self.index.values() {
            if let Some(entity) = world.exile(entity_id) {
                if let Some(terrain) = entity.get::<Terrain>() {
                    log::debug!(
                        "ECS_TERRAIN: exile {entity_id:?}, lod={}",
                        terrain.lod.value()
                    );
                } else {
                    log::error!("ECS_TERRAIN: exile {entity_id:?} ! not a terrain");
                }
            } else {
                log::error!("ECS_TERRAIN: exile {entity_id:?} -> failed");
            }
            tiles_to_exile.push(*entity_id);
        }
        let scene = index.values().copied().collect::<Vec<_>>();
        self.index = index;

        // log::debug!("tiles: to_exile - {}", tiles_to_exile.len());
        // log::debug!("tiles: to_spawn - {}", tiles_to_spawn.len());
        // log::debug!("tiles: total    - {}", self.index.len());

        SpawnTerrainOutput {
            tiles_to_exile,
            tiles_to_spawn,
            scene,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::SpawnTerrain;

    #[test]
    fn detect_tile_lod_by_axis() {
        let spawn_terrain = SpawnTerrain::default();
        let tiles_in_view_range = spawn_terrain.tiles_in_view_range as i32;

        assert_eq!(spawn_terrain.get_tile_lod(0, 0).value(), 0);
        assert_eq!(
            spawn_terrain.get_tile_lod(-tiles_in_view_range, 2).value(),
            2
        );
        assert_eq!(
            spawn_terrain
                .get_tile_lod(-tiles_in_view_range + 1, 1)
                .value(),
            1
        );
    }
}
