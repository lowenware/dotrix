use dotrix::assets::Shader;
use dotrix::camera;
use dotrix::prelude::*;
use dotrix::renderer::{
    BindGroup, Binding, PipelineLayout, PipelineOptions, Stage, StorageBuffer, UniformBuffer,
};
use dotrix::{Assets, Camera, Pipeline, Renderer, World};

const PARTICLES_COUNT: usize = 10;

struct Particle {
    params: UniformBuffer,
    transforms: StorageBuffer,
}

fn main() {
    Dotrix::application("Dotrix: Compute Example")
        .with(System::from(startup))
        .with(System::from(compute))
        .with(System::from(render))
        .run();
}

fn startup(
    mut camera: Mut<Camera>,
    mut world: Mut<World>,
    mut assets: Mut<Assets>,
    renderer: Const<Renderer>,
) {
    camera.target = [0., 0., 0.].into();
    camera.distance = 2.0;
    camera.xz_angle = 0.0;
}

fn compute(assets: Const<Assets>, world: Const<World>, mut renderer: Mut<Renderer>) {
    for (particle, pipeline) in world.query::<(&Particle, &mut Pipeline)>() {
        // Set the shader on the pipeline
        if pipeline.shader.is_null() {
            pipeline.shader = assets.find::<Shader>("compute").unwrap_or_default();
        }

        // Bind the uniforms to the shader
        if !pipeline.ready() {
            if let Some(shader) = assets.get(pipeline.shader) {
                if !shader.loaded() {
                    continue;
                }

                renderer.bind(
                    pipeline,
                    PipelineLayout {
                        label: "Compute Particles".into(),
                        mesh: None,
                        shader,
                        bindings: &[BindGroup::new(
                            "Globals",
                            vec![
                                Binding::Uniform("Params", Stage::Compute, &particle.params),
                                Binding::Storage(
                                    "Transforms",
                                    Stage::Compute,
                                    &particle.transforms,
                                ),
                            ],
                        )],
                        options: PipelineOptions::default(),
                    },
                );
            }
        }

        // Run the pipeline
        renderer.compute(pipeline, WorkGroups { x: 1, y: 1, z: 1 });
    }
}

fn render(mut world: Mut<World>, mut assets: Mut<Assets>, renderer: Const<Renderer>) {}
