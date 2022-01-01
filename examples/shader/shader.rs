use dotrix::assets::{Mesh, Shader};
use dotrix::camera::ProjView;
use dotrix::ecs::{Const, Mut, System};
use dotrix::renderer::{BindGroup, Binding, PipelineLayout, PipelineOptions, Stage, UniformBuffer};
use dotrix::{Application, Assets, Globals, Id, Pipeline, Renderer, World};

pub const PIPELINE_LABEL: &str = "example::gradient";

// This holds our public level representation of the data
#[derive(Clone, Copy)]
pub struct Gradient {
    pub nadir_color: [f32; 4],
    pub zenith_color: [f32; 4],
    pub mesh: Id<Mesh>,
    // Could add things like matrix transform too
}

// This is the low level representation of the uniform data
// that will go to the gpu
#[repr(C)]
#[derive(Default, Copy, Clone)]
struct GradientUniform {
    zenith_color: [f32; 4],
    nadir_color: [f32; 4],
    // Uniform data should be divisible by 16
    // so we add padding
    //
    // https://www.w3.org/TR/WGSL/#alignment-and-size
    padding: [f32; 8],
}

// This tranforms from the high level to the low level data
impl From<&Gradient> for GradientUniform {
    fn from(obj: &Gradient) -> Self {
        Self {
            zenith_color: obj.zenith_color,
            nadir_color: obj.nadir_color,
            ..Default::default() // Just use default to add in the padding
        }
    }
}

// Byte muck is used to cast the data into a byte array
unsafe impl bytemuck::Zeroable for GradientUniform {}
unsafe impl bytemuck::Pod for GradientUniform {}

/// This holds the gpu uniform buffer
#[derive(Default)]
pub struct GradientBuffer {
    pub uniform: UniformBuffer,
}

/// startup system
///
/// We add the uniform buffer at startup
/// where it will then be avaliable for the render to use (and reuse)
///
/// We also compile our shader
pub fn startup(mut globals: Mut<Globals>, renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    globals.set(GradientBuffer::default());

    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: include_str!("./shader.wgsl").to_string(),
        ..Default::default()
    };

    shader.load(&renderer);

    assets.store_as(shader, PIPELINE_LABEL);
}

/// render system
///
/// Here we update the contents of the uniform buffer
/// then render
pub fn render(
    world: Const<World>,
    mut renderer: Mut<Renderer>,
    mut globals: Mut<Globals>,
    assets: Const<Assets>,
) {
    for (gradient, pipeline) in world.query::<(&Gradient, &mut Pipeline)>() {
        // Update the uniform
        if let Some(uniform_buffer) = globals.get_mut::<GradientBuffer>() {
            let uniform: GradientUniform = gradient.into();
            renderer.load_uniform_buffer(
                &mut uniform_buffer.uniform,
                bytemuck::cast_slice(&[uniform]),
            );
        }

        // Get the uniform for render
        let gradient_buffer = globals
            .get::<GradientBuffer>()
            .expect("Gradient buffer must be loaded");

        // Get the uniform that holds the camera's project view
        let proj_view = globals
            .get::<ProjView>()
            .expect("ProjView buffer must be loaded");

        // Set the shader on the pipline
        if pipeline.shader.is_null() {
            pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
        }

        // check if model is disabled or already rendered
        if !pipeline.cycle(&renderer) {
            continue;
        }

        let mesh = assets.get(gradient.mesh).expect("Mesh must be loaded");

        // Bind the uniforms to the shader
        if !pipeline.ready() {
            if let Some(shader) = assets.get(pipeline.shader) {
                if !shader.loaded() {
                    continue;
                }

                renderer.bind(
                    pipeline,
                    PipelineLayout {
                        label: String::from(PIPELINE_LABEL),
                        mesh,
                        shader,
                        bindings: &[BindGroup::new(
                            "Globals",
                            vec![
                                Binding::Uniform("ProjView", Stage::Vertex, &proj_view.uniform),
                                Binding::Uniform(
                                    "Gradient",
                                    Stage::Fragment,
                                    &gradient_buffer.uniform,
                                ),
                            ],
                        )],
                        options: PipelineOptions::default(),
                    },
                );
            }
        }

        // Run the pipeline
        renderer.run(pipeline, mesh);
    }
}

pub fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(render));
}
