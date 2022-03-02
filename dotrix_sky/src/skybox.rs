//! Component and buffers

use dotrix_core::assets::{Mesh, Shader};
use dotrix_core::ecs::{Const, Mut, System};
use dotrix_core::renderer::{
    BindGroup, Binding, Buffer, DepthBufferMode, DrawArgs, PipelineLayout, Render, RenderOptions,
    Sampler, Stage,
};
use dotrix_core::{Application, Assets, Camera, CubeMap, Globals, Renderer, World};

use dotrix_math::Mat4;

pub const PIPELINE_LABEL: &str = "skybox";

/// SkyBox component
///
/// SkyBox is a cube with 6 textures on internal sides. It has one major difference from the rgular
/// cube though: SkyBox is fixed relatively to camera position.
///
/// Usage is quite straight forward. You need 6 textures and spawn an entity with the compomnet.
///
/// Dotrix provides a simple
/// [example](https://github.com/lowenware/dotrix/blob/main/examples/skybox/main.rs) of how to
/// use the [`SkyBox`].
pub struct SkyBox {
    pub view_range: f32,
    pub uniform: Buffer,
}

impl Default for SkyBox {
    fn default() -> Self {
        Self {
            view_range: 300.0,
            uniform: Buffer::uniform("SkyBox Buffer"),
        }
    }
}

/// Skybox startup system
pub fn startup(mut assets: Mut<Assets>, renderer: Const<Renderer>) {
    // generate mesh
    let mut mesh = Mesh::default();
    mesh.with_vertices(&[
        // front
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
        // top
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        // right
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [1.0, 1.0, 1.0],
        [1.0, -1.0, 1.0],
        // left
        [-1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, -1.0],
        // back
        [-1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0],
        [1.0, -1.0, -1.0],
        [-1.0, -1.0, -1.0],
        // bottom
        [1.0, -1.0, 1.0],
        [-1.0, -1.0, 1.0],
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
    ]);
    mesh.with_indices(&[
        2, 1, 0, 0, 3, 2, 6, 5, 4, 4, 7, 6, 10, 9, 8, 8, 11, 10, 14, 13, 12, 12, 15, 14, 18, 17,
        16, 16, 19, 18, 22, 21, 20, 20, 23, 22,
    ]);

    mesh.load(&renderer);

    assets.store_as(mesh, PIPELINE_LABEL);

    // prepare shader
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: String::from(include_str!("shaders/skybox.wgsl")),
        ..Default::default()
    };

    shader.load(&renderer);

    assets.store_as(shader, PIPELINE_LABEL);
}

/// SkyBox rendering system
pub fn render(
    mut renderer: Mut<Renderer>,
    mut assets: Mut<Assets>,
    camera: Const<Camera>,
    globals: Const<Globals>,
    world: Const<World>,
) {
    let query = world.query::<(&mut SkyBox, &mut CubeMap, &mut Render)>();

    for (skybox, cubemap, render) in query {
        let proj_view_mx = camera.proj.as_ref().unwrap() * camera.view_matrix_static();
        let scale = if skybox.view_range > 0.1 {
            skybox.view_range
        } else {
            1.0
        };
        let uniform = Uniform {
            proj_view: proj_view_mx.into(),
            scale: Mat4::from_scale(scale).into(),
        };

        if render.pipeline.shader.is_null() {
            render.pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
        }

        // check if model is disabled or already rendered
        if !render.pipeline.cycle(&renderer) {
            continue;
        }

        renderer.load_buffer(&mut skybox.uniform, bytemuck::cast_slice(&[uniform]));

        if !cubemap.load(&renderer, &mut assets) {
            continue;
        }

        let mesh = assets
            .get(
                assets
                    .find::<Mesh>(PIPELINE_LABEL)
                    .expect("SkyBox must be initialized with the `skybox::startup` system"),
            )
            .unwrap();

        if !render.pipeline.ready(&renderer) {
            if let Some(shader) = assets.get(render.pipeline.shader) {
                let sampler = globals
                    .get::<Sampler>()
                    .expect("Sampler buffer must be loaded");

                renderer.bind(
                    &mut render.pipeline,
                    PipelineLayout::Render {
                        label: String::from(PIPELINE_LABEL),
                        mesh,
                        shader,
                        bindings: &[
                            BindGroup::new(
                                "Globals",
                                vec![
                                    Binding::Uniform("SkyBox", Stage::Vertex, &skybox.uniform),
                                    Binding::Sampler("Sampler", Stage::Fragment, sampler),
                                ],
                            ),
                            BindGroup::new(
                                "Locals",
                                vec![Binding::Texture3D(
                                    "CubeMap",
                                    Stage::Fragment,
                                    &cubemap.buffer,
                                )],
                            ),
                        ],
                        options: RenderOptions {
                            depth_buffer_mode: DepthBufferMode::ReadWrite,
                            ..Default::default()
                        },
                    },
                );
            }
        }

        renderer.draw(&mut render.pipeline, mesh, &DrawArgs::default());
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct Uniform {
    proj_view: [[f32; 4]; 4],
    scale: [[f32; 4]; 4],
}

unsafe impl bytemuck::Zeroable for Uniform {}
unsafe impl bytemuck::Pod for Uniform {}

pub fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(render));
}
