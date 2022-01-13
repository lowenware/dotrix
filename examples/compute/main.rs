use rand::distributions::{Distribution, Uniform};
use rand::SeedableRng;

use dotrix::assets::Shader;
use dotrix::camera;
use dotrix::camera::ProjView;
use dotrix::egui::{self, Egui};
use dotrix::overlay::{self, Overlay};
use dotrix::prelude::*;
use dotrix::primitives::Cube;
use dotrix::renderer::{
    BindGroup, Binding, Options, PipelineLayout, PipelineOptions, Stage, StorageBuffer,
    UniformBuffer, WorkGroups,
};
use dotrix::{Assets, Camera, Color, Frame, Globals, Pipeline, Renderer, World};

const PARTICLES_COUNT: usize = 2000;
const PARTICLES_PER_WORKGROUP: usize = 32;
const COMPUTE_PIPELINE: &str = "dotrix::example::compute";
const RENDER_PIPELINE: &str = "dotrix::example::render";
const PARTICLE_MESH: &str = "dotrix::example::particle";

#[derive(Default)]
struct Compute {
    pipeline: Pipeline,
}

#[derive(Default)]
struct ParticlesSpawner {
    params: UniformBuffer,
    particles: StorageBuffer,
    work_group_size: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Params {
    position: [f32; 3],
    simulation_time: f32,
    gravity: f32,
    start_color: [f32; 4],
    end_color: [f32; 4],
    padding: [f32; 3],
}

unsafe impl bytemuck::Zeroable for Params {}
unsafe impl bytemuck::Pod for Params {}

impl Default for Params {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            simulation_time: 0.0,
            gravity: -0.002,
            start_color: [0.2, 0., 0., 1.],
            end_color: [0.8, 0.3, 0.3, 1.],
            padding: [0.0; 3],
        }
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
struct Particle {
    position: [f32; 4],
    velocity: [f32; 4],
    color: [f32; 4],
    life_time: f32,
    padding: [f32; 3],
}

unsafe impl bytemuck::Zeroable for Particle {}
unsafe impl bytemuck::Pod for Particle {}

fn main() {
    Dotrix::application("Dotrix: Compute Example")
        .with(overlay::extension)
        .with(egui::extension)
        .with(System::from(startup))
        .with(System::from(compute))
        .with(System::from(render))
        .with(System::from(ui))
        .with(System::from(camera::control))
        .run();
}

fn startup(
    mut assets: Mut<Assets>,
    mut camera: Mut<Camera>,
    mut world: Mut<World>,
    mut renderer: Mut<Renderer>,
) {
    // set up clear color
    renderer.set_clear_color(Color::rgb(0.01, 0.01, 0.01));
    // set up camera
    camera.target = [0., 0., 0.].into();
    camera.distance = 2.0;
    camera.xz_angle = 0.0;
    camera.y_angle = std::f32::consts::PI / 2.0;

    // set up particles spawner
    let mut spawner = ParticlesSpawner {
        particles: StorageBuffer::new_readwrite(),
        work_group_size: (PARTICLES_COUNT as f32 / PARTICLES_PER_WORKGROUP as f32).ceil() as u32,
        ..Default::default()
    };
    let mut particles = vec![Particle::default(); PARTICLES_COUNT];

    // set initial particles data
    let mut rng = rand::rngs::StdRng::seed_from_u64(16);
    let unif = Uniform::new_inclusive(-1.0, 1.0);
    for particle in particles.iter_mut() {
        particle.velocity[0] = 0.1 * unif.sample(&mut rng); // X
        particle.velocity[1] = 0.1 * (unif.sample(&mut rng) + 2.0) / 2.0; // Y
        particle.velocity[2] = 0.1 * unif.sample(&mut rng); // Z
        particle.life_time = 1.5 + unif.sample(&mut rng); // life time
    }

    renderer.load_storage_buffer(
        &mut spawner.particles,
        bytemuck::cast_slice(particles.as_slice()),
    );

    world.spawn(Some((
        spawner,
        Compute::default(),
        Pipeline {
            options: Options {
                start_index: 0,
                end_index: PARTICLES_COUNT as u32,
                ..Default::default()
            },
            ..Default::default()
        },
    )));

    // load shaders
    let shaders = [
        (COMPUTE_PIPELINE, include_str!("./compute.wgsl")),
        (RENDER_PIPELINE, include_str!("./render.wgsl")),
    ];
    for (pipeline_name, pipeline_code) in shaders.iter() {
        let mut shader = Shader {
            name: String::from(*pipeline_name),
            code: String::from(*pipeline_code),
            ..Default::default()
        };
        shader.load(&renderer);
        assets.store_as(shader, pipeline_name);
    }

    // add mesh
    let mut mesh = Cube::builder(0.01).with_positions().mesh();
    mesh.load(&renderer);
    assets.store_as(mesh, PARTICLE_MESH);
}

fn compute(
    assets: Const<Assets>,
    frame: Const<Frame>,
    world: Const<World>,
    mut renderer: Mut<Renderer>,
) {
    for (spawner, compute) in world.query::<(&mut ParticlesSpawner, &mut Compute)>() {
        // Set the shader on the pipeline
        if compute.pipeline.shader.is_null() {
            compute.pipeline.shader = assets.find::<Shader>(COMPUTE_PIPELINE).unwrap_or_default();
        }
        // update time delta
        let params = Params {
            simulation_time: frame.time().as_secs_f32(),
            ..Default::default()
        };
        renderer.load_uniform_buffer(&mut spawner.params, bytemuck::cast_slice(&[params]));

        // Bind the uniforms to the shader
        if !compute.pipeline.ready() {
            if let Some(shader) = assets.get(compute.pipeline.shader) {
                if !shader.loaded() {
                    continue;
                }

                renderer.bind(
                    &mut compute.pipeline,
                    PipelineLayout {
                        label: "Compute Particles".into(),
                        mesh: None,
                        shader,
                        bindings: &[BindGroup::new(
                            "Globals",
                            vec![
                                Binding::Uniform("Params", Stage::Compute, &spawner.params),
                                Binding::Storage("Particles", Stage::Compute, &spawner.particles),
                            ],
                        )],
                        options: PipelineOptions::default(),
                    },
                );
            }
        }

        // Run the pipeline
        let work_group_size = spawner.work_group_size;
        renderer.compute(
            &mut compute.pipeline,
            WorkGroups {
                x: work_group_size,
                y: 1,
                z: 1,
            },
        );
    }
}

fn render(
    globals: Const<Globals>,
    world: Const<World>,
    assets: Const<Assets>,
    mut renderer: Mut<Renderer>,
) {
    for (spawner, pipeline) in world.query::<(&mut ParticlesSpawner, &mut Pipeline)>() {
        // Set the shader on the pipeline
        if pipeline.shader.is_null() {
            pipeline.shader = assets.find::<Shader>(RENDER_PIPELINE).unwrap_or_default();
        }

        let mesh = assets
            .get(assets.find(PARTICLE_MESH).expect("Mesh must be loaded"))
            .unwrap();

        if !pipeline.ready() {
            if let Some(shader) = assets.get(pipeline.shader) {
                if !shader.loaded() {
                    continue;
                }

                // Get the uniform that holds the camera's project view
                let proj_view = globals
                    .get::<ProjView>()
                    .expect("ProjView buffer must be loaded");

                renderer.bind(
                    pipeline,
                    PipelineLayout {
                        label: String::from(RENDER_PIPELINE),
                        mesh: Some(mesh),
                        shader,
                        bindings: &[BindGroup::new(
                            "Globals",
                            vec![
                                Binding::Uniform("ProjView", Stage::Vertex, &proj_view.uniform),
                                Binding::Storage("Particles", Stage::Vertex, &spawner.particles),
                            ],
                        )],
                        options: PipelineOptions::default(),
                    },
                );
            }
        }

        renderer.run(pipeline, mesh);
    }
}

fn ui(overlay: Const<Overlay>, frame: Const<Frame>) {
    let egui_overlay = overlay
        .get::<Egui>()
        .expect("Egui overlay must be added on startup");

    egui::Area::new("FPS counter")
        .fixed_pos(egui::pos2(16.0, 16.0))
        .show(&egui_overlay.ctx, |ui| {
            ui.colored_label(
                egui::Rgba::from_rgb(255.0, 255.0, 255.0),
                format!("FPS: {:.1}", frame.fps()),
            );
        });
}
