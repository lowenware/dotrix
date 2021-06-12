use std::{
    collections::HashMap,
};

use dotrix_core::{
    components::Model,
    ecs::{ Const, Mut, Context },
    services::{ Assets, Camera, World },
};

use crate::{ Terrain, Manager };

/// Terrain spawn system context
#[derive(Default)]
pub struct Spawner {
    tiles: HashMap<TileIndex, TileState>,
    last_viewer_position: Option<[f32; 2]>,
}

#[derive(Default)]
struct TileState {
    lod: usize,
    visible: bool,
    spawned: bool,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct TileIndex {
    x: i32,
    z: i32,
}

struct Viewer {
    position: [f32; 2],
    view_distance_sq: f32,
}

/// Terrain spawn system
/// Controls presense of terrain tiles, generation of meshes, and resource releasing
pub fn spawn(
    mut ctx: Context<Spawner>,
    mut manager: Mut<Manager>,
    camera: Const<Camera>,
    mut assets: Mut<Assets>,
    mut world: Mut<World>,
) {

    let view_distance = manager.view_distance;
    // get viewer
    let viewer = Viewer {
        view_distance_sq: view_distance * view_distance,
        position: [camera.target.x, camera.target.z],
    };

    // check if update is necessary
    if let Some(last_viewer_position) = ctx.last_viewer_position.as_ref() {
        let dx = viewer.position[0] - last_viewer_position[0];
        let dz = viewer.position[1] - last_viewer_position[1];
        if  !manager.force_spawn && dx * dx + dz * dz < manager.spawn_if_moved_by {
            return;
        }
    }
    ctx.last_viewer_position = Some(viewer.position);

    // disable force spawn if was enabled previously
    manager.force_spawn = false;

    // mark all tiles non visible
    for tile in ctx.tiles.values_mut() {
        tile.visible = false;
    }

    // calculate terrain tiles that has to be visible
    let max_lod = manager.max_lod;
    let tile_size = manager.tile_size as f32 * (2.0_f32).powf(max_lod as f32);
    let tiles_per_view_distance = (view_distance / tile_size as f32).ceil() as i32;
    let half_tile_size = tile_size as i32 / 2;
    let from_x = ((viewer.position[0] / tile_size).floor() * tile_size) as i32;
    let from_z = ((viewer.position[1] / tile_size).floor() * tile_size) as i32;

    for zi in -tiles_per_view_distance..tiles_per_view_distance {
        let z = from_z + zi * tile_size as i32 + half_tile_size as i32;
        for xi in -tiles_per_view_distance..tiles_per_view_distance {
            let x = from_x + xi * tile_size as i32 + half_tile_size as i32;
            // recursively calculate what lods should be spawned and spawn them
            queue_tiles_to_spawn(&mut ctx, &viewer, half_tile_size, max_lod, TileIndex {x, z});
        }
    }

    // exile tiles
    /*
    let query = world.query::<(Terrain, Entity)>();
    for (terrain, entity) in query {
        let index = TileIndex { x: terrain.x, z: terrain.z };
        if let Some(tile) = ctx.tiles.get(&index) {
            if !tile.visible {
                world.exile(entity);
            }
        }
    }

    // cleanup tiles registry of the exiled tiles
    ctx.tiles.retain(|_, tile| tile.visible);
    */

    // spawn missing tiles
    for (index, tile) in ctx.tiles.iter_mut() {
        if tile.spawned {
            continue;
        }

        let terrain = Terrain {
            x: index.x,
            z: index.z,
            lod: tile.lod,
        };
        let mesh = manager.generate_mesh(&terrain);
        let model = Model {
            mesh: assets.store(mesh),
            texture: manager.texture,
            ..Default::default()
        };

        world.spawn(Some((model, terrain)));

        tile.spawned = true;
    }


    /*
     * println!("Tiles to spawn: {}", ctx.tiles.len());
    for (index, tile) in ctx.tiles.iter() {
        println!("   - Tile({}) @ {}:{}, flags = {}/{}", tile.lod, index.x, index.z, tile.visible, tile.spawned);
    }
    */


}

fn queue_tiles_to_spawn(
    ctx: &mut Spawner,
    viewer: &Viewer,
    half_tile_size: i32,
    lod: usize,
    position: TileIndex,
) {
    let dx = position.x as f32 - viewer.position[0];
    let dz = position.z as f32 - viewer.position[1];
    let distance_sq = dx * dx + dz * dz;
    let lod_distance_sq = (4 * half_tile_size * half_tile_size) as f32;
    let x = position.x;
    let z = position.z;

    if distance_sq > lod_distance_sq || lod == 0 {
        if distance_sq > viewer.view_distance_sq {
            return; // the tile is out of the view distance range
        }
        let mut tile = ctx.tiles
            .entry(position)
            .or_insert(TileState { lod, ..Default::default() });
        tile.visible = true;
    } else {
        // Higher lod is required
        let half_tile_size = half_tile_size / 2;
        let x1 = x - half_tile_size as i32;
        let x2 = x + half_tile_size as i32;
        let z1 = z - half_tile_size as i32;
        let z2 = z + half_tile_size as i32;
        let higher_lod_tiles = [
            TileIndex { x: x1, z: z1 },
            TileIndex { x: x2, z: z1 },
            TileIndex { x: x1, z: z2 },
            TileIndex { x: x2, z: z2 },
        ];

        for tile in higher_lod_tiles.iter() {
            queue_tiles_to_spawn(ctx, viewer, half_tile_size, lod - 1, *tile);
        }
    }
}
