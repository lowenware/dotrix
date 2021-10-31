use std::collections::HashMap;
use dotrix::prelude::*;
use dotrix::{ Assets, Camera, Color, World };
use dotrix::assets::Texture;
use dotrix::pbr::Light;
use dotrix::math::Vec3;
use dotrix::terrain::{ Map, HeightMap, Lod, Simple as SimpleTerrain };

pub use dotrix::terrain::extension;

use crate::ui::{ Controls, HeightMapAction };

const COMPONENT_SIZE: usize = 128;

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

    // TODO: remove it together with the asset
    // assets.import("assets/terrain/heightmap.png");
}

#[derive(Default)]
pub struct Loader {
    loaded: bool,
}

pub fn load(
    mut context: Context<Loader>,
    mut controls: Mut<Controls>,
    mut assets: Mut<Assets>,
    mut map: Mut<Map>,
) {
    // resize terrain map
    if controls.terrain.map_reload {
        let lod = Lod::from_level(controls.terrain.lod);
        let scale = lod.scale();
        let units_per_side = controls.terrain.component.units_per_side() as u32;
        let tiles_per_x = controls.terrain.tiles_per_x;
        // Note: Only square terrain is implemented
        let size_x = scale * controls.terrain.tiles_per_x * units_per_side + 1;
        // let size_z = controls.terrain.tiles_per_z * units_per_side + 1;

        map.component = controls.terrain.component;
        map.view_range = tiles_per_x / 2;
        map.min_lod = lod;
        map.unit_size = 1.0;

        println!("HeightMap: Tiles: {}, Size: {}, view_range: {}", controls.terrain.tiles_per_x,
            size_x, map.view_range);

        map.generator.set_y_scale(controls.terrain.y_scale);
        map.generator.resize(size_x, size_x);
        map.generator.set_offset(-(size_x as i32 / 2), -(size_x as i32 / 2));
        map.set_dirty();
        controls.terrain.map_reload = false;
    }

    if let Some(height_map_action) = controls.terrain.height_map_action.take() {
        match height_map_action {
            HeightMapAction::Reset => {
                println!("New Height Map");
                map.generator.reset();
                map.set_dirty();
            },
            HeightMapAction::Import => {
                println!("Import Height Map");
            },
            HeightMapAction::Export => {
                let heightmap = map.generator.export("./heightmap-export.png");
                println!("Export Height Map");
            }
        }
    }

    /*
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
    */
}

