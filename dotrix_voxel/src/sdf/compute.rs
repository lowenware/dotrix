use super::*;
use crate::Grid;
use dotrix_core::{
    assets::Shader,
    ecs::{Const, Mut, System},
    Application, Assets, Globals, Renderer, World,
};

const PIPELINE_LABEL: &str = "dotrix_voxel::render::sdf_compute";

fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: String::from(include_str!("./compute.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, PIPELINE_LABEL);
}

fn compute(
    world: Const<World>,
    assets: Const<Assets>,
    globals: Const<Globals>,
    mut renderer: Mut<Renderer>,
) {
    for (grid, sdf) in world.query::<(&Grid, &mut ComputeSdf)>() {
        if sdf.compute_pipeline.shader.is_null() {
            sdf.compute_pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
        }

        if !pipeline.cycle(&renderer) {
            return;
        }
        let mesh = assets
            .get(
                assets
                    .find::<Mesh>(PIPELINE_LABEL)
                    .expect("Sdf mesh must be initialized with the dotrix_voxel startup system"),
            )
            .unwrap();
        if sdf.load(grid, transform, &renderer) {
            // If loading returns true then the 3d texture was updated in such a way that we
            // to recompute the SDF
            sdf.compute_pipeline.bindings.unload();
        }

        renderer.compute(
            &mut shadow_trace.pipeline,
            WorkGroups {
                x: workgroup_size_x,
                y: workgroup_size_y,
                z: 1,
            },
        );
    }
}

pub fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(compute));
}
