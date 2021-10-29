use dotrix_core::{ Assets, Camera, Color, Globals, Id, Pipeline, World };
use dotrix_core::ecs::{ Entity, Mut, Const, Context };
use dotrix_core::assets::{ Mesh, Shader };
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
use dotrix_math::Vec3;
use dotrix_pbr::{ Material, Lights };

use crate::{ Component, Map, Layers, Lod, Node, Terrain, VecXZ };

const PIPELINE_LABEL: &str = "dotrix::terrain";

#[derive(Default)]
pub struct Spawner {
    last_viewer_position: Option<Vec3>,
}

pub fn spawn(
    mut ctx: Context<Spawner>,
    mut map: Mut<Map>,
    mut assets: Mut<Assets>,
    mut world: Mut<World>,
    camera: Const<Camera>,
) {
    if !map.generator.dirty() {
        return;
    }

    let viewer = VecXZ::new(
        camera.target.x,
        camera.target.z,
    );

    let lod = map.min_lod;

    let max_component_size = map.component_size(lod);
    let view_range = map.view_range as i32;

    // Get index of the component (lowest LOD) where the viewer is
    let index_x = (viewer.x / max_component_size).floor() as i32;
    let index_z = (viewer.z / max_component_size).floor() as i32;

    map.request_cleanup();

    for x in -view_range..view_range {
        let x_pos = Component::axis_offset(index_x + x, lod);
        for z in -view_range..view_range {
            let z_pos = Component::axis_offset(index_z + z, lod);
            // position is the map coordinate of the terrain chunk center
            let position = VecXZ::new(x_pos, z_pos);
            check_lod_and_spawn(
                &viewer,
                &mut map,
                &mut assets,
                &mut world,
                position,
                lod,
            );
        }
    }

    let query = world.query::<(&Entity, &Terrain)>();
    for (entity, terrain) in query {
        if let Some(node) = map.nodes.get_mut(&terrain.position) {
            node.entity = Some(*entity);
        }
    }

    map.nodes.retain(|position, node| {
        if !node.cleanup {
            return true;
        }

        if let Some(entity) = node.entity.take() {
            world.exile(entity);
        }
        assets.remove(node.mesh);
        false
    });

    map.generator.set_dirty(false);
}

fn check_lod_and_spawn(
    _viewer: &VecXZ<f32>,
    map: &mut Map,
    assets: &mut Assets,
    world: &mut World,
    position: VecXZ<i32>,
    lod: Lod,
) {
    // TODO: check LOD and spawn terrain by recursive calls

    let scale = lod.scale();
    if let Some(node) = map.nodes.get_mut(&position) {
        node.cleanup = false;
        if node.dirty {
            if let Some(new_mesh) = map.generator.get(map.component, position, scale, map.unit_size) {
                if let Some(mesh) = assets.get_mut(node.mesh) {
                    mesh.vertices = new_mesh.vertices;
                    // wgpu panics on buffer rewriting. why?
                    // mesh.changed = true;
                    mesh.unload();
                }
            }
            println!("redraw {}:{}", position.x, position.z);
            node.dirty = false;
        }
    } else {
        if let Some(mesh) = map.generator.get(map.component, position, scale, map.unit_size) {
            let mesh = assets.store(mesh);
            let terrain = Terrain {
                position,
                scale,
                mesh,
                loaded: false,
            };
            let material = Material {
                // texture: terrain.texture,
                albedo: match position {
                    VecXZ { x:-24, z:-24} => Color::red(),
                    // 1 => Color::red(),
                    // 2 => Color::green(),
                    // 4 => Color::blue(),
                    _ => Color::grey()
                },
                ..Default::default()
            };
            let pipeline = Pipeline::default();

            world.spawn(Some((terrain, material, pipeline)));

            map.nodes.insert(position, Node {
                lod,
                mesh,
                ..Default::default()
            });
        }
    }
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

/// Terrain rendering system
pub fn render(
    mut renderer: Mut<Renderer>,
    mut assets: Mut<Assets>,
    globals: Const<Globals>,
    world: Const<World>,
) {
    let mut count = 0;
    let query = world.query::<(
        &mut Terrain,
        &mut Material,
        &mut Pipeline
    )>();

    let begin = std::time::Instant::now();

    for (terrain, material, pipeline) in query {
        count += 1;

        if pipeline.shader.is_null() {
            pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL)
                .unwrap_or_default();
        }

        // check if model is disabled or already rendered
        if !pipeline.cycle(&renderer) { continue; }

        //if !terrain.loaded {
            if let Some(mesh) = assets.get_mut(terrain.mesh) {
                mesh.load(&renderer);
            }
        //    terrain.loaded = true;
        //}

        if !material.load(&renderer, &mut assets) { continue; }

        let mesh = assets.get(terrain.mesh).unwrap();

        if !pipeline.ready() {
            if let Some(shader) = assets.get(pipeline.shader) {
                if !shader.loaded() { continue; }
                println!("Init terrain pipeline {} with scale {}", count, terrain.scale);

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
                            Binding::Uniform("Material", Stage::Fragment, &material.uniform),
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

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn it_works() {
	}
}
