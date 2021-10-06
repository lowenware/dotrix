use std::{
    collections::HashMap,
};

use dotrix_core::{ Color, Id, Pipeline, Camera, Globals, World };
use dotrix_core::assets::{ Assets, Mesh, Shader };
use dotrix_core::ecs::{ Entity, Mut, Const, Context };
use dotrix_core::camera::ProjView;
use dotrix_core::renderer::{
    BindGroup,
    Binding,
    PipelineLayout,
    PipelineOptions,
    Renderer,
    Sampler,
    Stage,
};

use dotrix_pbr::{ Material, Lights };

use crate::{ Terrain, Tile, Layers };

const PIPELINE_LABEL: &str = "dotrix::terrain";

/// Terrain spawn system context
#[derive(Default)]
pub struct Spawner {
    tiles: HashMap<TileIndex, TileState>,
    last_viewer_position: Option<[f32; 2]>,
    to_exile: Vec<(Entity, Id<Mesh>)>,
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

/// Terrain Startup System
pub fn startup(
    mut assets: Mut<Assets>,
    mut globals: Mut<Globals>,
    renderer: Const<Renderer>,
) {
    // prepare layers
    let mut layers = Layers::default();
    layers.load(&renderer);
    globals.set(layers);

    // prepare shader
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: Lights::add_to_shader(include_str!("shaders/terrain.wgsl"), 0, 2),
        ..Default::default()
    };
    shader.load(&renderer);
    assets.store_as(shader, PIPELINE_LABEL);
}

/// Terrain spawn system
/// Controls presense of terrain tiles, generation of meshes, and resource releasing
pub fn spawn(
    mut ctx: Context<Spawner>,
    mut terrain: Mut<Terrain>,
    camera: Const<Camera>,
    mut assets: Mut<Assets>,
    mut world: Mut<World>,
) {
    let view_distance = terrain.view_distance;
    // get viewer
    let viewer = Viewer {
        view_distance_sq: view_distance * view_distance,
        position: [camera.target.x, camera.target.z],
    };

    // check if update is necessary
    if let Some(last_viewer_position) = ctx.last_viewer_position.as_ref() {
        let dx = viewer.position[0] - last_viewer_position[0];
        let dz = viewer.position[1] - last_viewer_position[1];
        if  !terrain.force_spawn && dx * dx + dz * dz < terrain.spawn_if_moved_by {
            return;
        }
    }
    ctx.last_viewer_position = Some(viewer.position);

    if terrain.force_spawn {
        ctx.tiles.clear();

        let query = world.query::<(&Tile, &mut Pipeline)>();
        for (_, pipeline) in query {
            pipeline.disabled = true;
        }

        terrain.force_spawn = false;
    }

    // mark all tiles non visible
    for tile in ctx.tiles.values_mut() {
        tile.visible = false;
    }

    // calculate terrain tiles that has to be visible
    let max_lod = terrain.max_lod;
    let tile_size = terrain.tile_size as f32 * (2.0_f32).powf(max_lod as f32);
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
    let query = world.query::<(&Tile, &Entity)>();
    for (tile, entity) in query {
        let index = TileIndex { x: tile.x, z: tile.z };
        let do_exile = if let Some(tile) = ctx.tiles.get_mut(&index) {
            !tile.visible
        } else {
            true
        };
        if do_exile {
            ctx.to_exile.push((*entity, tile.mesh));
        }
    }

    for (entity, mesh) in ctx.to_exile.iter() {
        world.exile(*entity);
        assets.remove(*mesh);
    }
    ctx.to_exile.clear();

    // cleanup tiles registry of the exiled tiles
    ctx.tiles.retain(|_, tile| tile.visible);

    // spawn missing tiles
    for (index, tile_state) in ctx.tiles.iter_mut() {
        if tile_state.spawned {
            continue;
        }

        let x = index.x;
        let z = index.z;
        let lod = tile_state.lod;

        let mesh = terrain.generate_tile_mesh(x, z, lod);
        let tile = Tile {
            x,
            z,
            lod,
            mesh: assets.store(mesh),
            loaded: false
        };
        let material = Material {
            texture: terrain.texture,
            albedo: Color::white(),
            ..Default::default()
        };
        let pipeline = Pipeline::default();

        world.spawn(Some((tile, material, pipeline)));

        tile_state.spawned = true;
    }
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

/// Terrain rendering system
pub fn render(
    mut renderer: Mut<Renderer>,
    mut assets: Mut<Assets>,
    globals: Const<Globals>,
    world: Const<World>,
) {
    let query = world.query::<(
        &mut Tile,
        &mut Material,
        &mut Pipeline
    )>();

    for (tile, material, pipeline) in query {

        if pipeline.shader.is_null() {
            pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL)
                .unwrap_or_else(Id::default);
        }

        // check if model is disabled or already rendered
        if !pipeline.cycle(&renderer) { continue; }

        if !tile.loaded {
            if let Some(mesh) = assets.get_mut(tile.mesh) {
                mesh.load(&renderer);
            }
            tile.loaded = true;
        }

        if !material.load(&renderer, &mut assets) { continue; }

        let mesh = assets.get(tile.mesh).unwrap();

        if !pipeline.ready() {
            if let Some(shader) = assets.get(pipeline.shader) {
                if !shader.loaded() { continue; }

                let texture = assets.get(material.texture).unwrap();

                let proj_view = globals.get::<ProjView>()
                    .expect("ProjView buffer must be loaded");

                let sampler = globals.get::<Sampler>()
                    .expect("ProjView buffer must be loaded");

                let lights = globals.get::<Lights>()
                    .expect("Lights buffer must be loaded");

                let layers = globals.get::<Layers>()
                    .expect("Terrain layers must be loaded");

                renderer.bind(pipeline, PipelineLayout {
                    label: String::from(PIPELINE_LABEL),
                    mesh,
                    shader,
                    bindings: &[
                        BindGroup::new("Globals", vec![
                            Binding::Uniform("ProjView", Stage::Vertex, &proj_view.uniform),
                            Binding::Sampler("Sampler", Stage::Fragment, sampler),
                            Binding::Uniform("Lights", Stage::Fragment, &lights.uniform),
                            Binding::Uniform("Layers", Stage::Fragment, &layers.uniform),
                        ]),
                        BindGroup::new("Locals", vec![
                            Binding::Uniform("Material", Stage::Vertex, &material.uniform),
                            Binding::Texture("Texture", Stage::Fragment, &texture.buffer),
                        ])
                    ],
                    options: PipelineOptions::default()
                });
            }
        }

        renderer.run(pipeline, mesh);
    }
}
