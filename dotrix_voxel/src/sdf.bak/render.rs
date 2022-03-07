use super::{CameraBuffer, LightStorageBuffer};
use crate::{ComputeSdf, Grid, RenderSdf};
use dotrix_core::{
    assets::{Mesh, Shader},
    ecs::{Const, Mut, System},
    renderer::{BindGroup, Binding, PipelineLayout, PipelineOptions, Sampler, Stage},
    Application, Assets, Globals, Renderer, Transform, World,
};
use dotrix_primitives::Cube;

const PIPELINE_LABEL: &str = "dotrix_voxel::render::sdf_render";

fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: String::from(include_str!("./render.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, PIPELINE_LABEL);

    let mut mesh = Cube::builder(1.0).with_positions().mesh();
    mesh.load(&renderer);
    assets.store_as(mesh, PIPELINE_LABEL);
}

pub fn render(
    mut renderer: Mut<Renderer>,
    world: Const<World>,
    assets: Const<Assets>,
    globals: Const<Globals>,
) {
    let camera_buffer = globals
        .get::<CameraBuffer>()
        .expect("ProjView buffer must be loaded");

    for (grid, render_sdf, compute_sdf, transform) in
        world.query::<(&Grid, &mut RenderSdf, &ComputeSdf, &Transform)>()
    {
        if render_sdf.pipeline.shader.is_null() {
            render_sdf.pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
        }
        if !render_sdf.pipeline.cycle(&renderer) {
            return;
        }
        let mesh = assets
            .get(
                assets
                    .find::<Mesh>(PIPELINE_LABEL)
                    .expect("Sdf mesh must be initialized with the dotrix_voxel startup system"),
            )
            .unwrap();
        render_sdf.load(grid, transform, &renderer);
        if !render_sdf.pipeline.ready() {
            let sampler = globals.get::<Sampler>().expect("Sampler must be loaded");

            let lights_buffer = globals
                .get::<LightStorageBuffer>()
                .expect("Sdfs buffer must be loaded");

            if let Some(shader) = assets.get(render_sdf.pipeline.shader) {
                renderer.bind(
                    &mut render_sdf.pipeline,
                    PipelineLayout {
                        label: String::from(PIPELINE_LABEL),
                        mesh: Some(mesh),
                        shader,
                        bindings: &[
                            BindGroup::new(
                                "Globals",
                                vec![
                                    Binding::Uniform("Camera", Stage::All, &camera_buffer.uniform),
                                    Binding::Sampler("Sampler", Stage::Fragment, sampler),
                                    Binding::Storage(
                                        "Lights",
                                        Stage::Fragment,
                                        &lights_buffer.storage,
                                    ),
                                ],
                            ),
                            BindGroup::new(
                                "Locals",
                                vec![
                                    Binding::Uniform("Sdf", Stage::All, &render_sdf.sdf_buffer),
                                    Binding::Texture3D(
                                        "Sdf",
                                        Stage::All,
                                        &compute_sdf.sdf_texture.as_ref().unwrap().buffer,
                                    ),
                                ],
                            ),
                        ],
                        options: PipelineOptions::default(),
                    },
                );
            }
        }
        // println!("Run Pipeline");
        renderer.run(&mut render_sdf.pipeline, mesh);
    }
}

pub fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(render));
}
