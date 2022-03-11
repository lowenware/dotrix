use crate::Grid;
use crate::TexSdf;
use dotrix_core::{
    assets::{Mesh, Shader},
    ecs::{Const, Mut, System},
    renderer::{BindGroup, Binding, PipelineLayout, RenderOptions, Sampler, Stage},
    Application, Assets, Globals, Renderer, World,
};
use dotrix_math::*;
use dotrix_primitives::Cube;

mod camera;
mod lights;

use camera::CameraBuffer;
pub use lights::Light;
use lights::LightStorageBuffer;

const PIPELINE_LABEL: &str = "dotrix_voxel::sdf::circle_trace";

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct SdfBufferData {
    // This transform scales the 1x1x1 cube so that it totally encloses the
    // voxels
    pub cube_transform: [[f32; 4]; 4],
    // Inverse fo cube_transform
    pub inv_cube_transform: [[f32; 4]; 4],
    // World transform of the voxel grid
    pub world_transform: [[f32; 4]; 4],
    // Inverse of world_transform
    pub inv_world_transform: [[f32; 4]; 4],
    // Dimensions of the voxel
    pub grid_dimensions: [f32; 3],
    pub padding: [f32; 1],
}

unsafe impl bytemuck::Zeroable for SdfBufferData {}
unsafe impl bytemuck::Pod for SdfBufferData {}

pub fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: String::from(include_str!("./circle_trace/render.wgsl")),
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

    for (grid, sdf) in world.query::<(&Grid, &mut TexSdf)>() {
        if sdf.pipeline.shader.is_null() {
            sdf.pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
        }
        if !sdf.pipeline.cycle(&renderer) {
            return;
        }
        let mesh = assets
            .get(
                assets
                    .find::<Mesh>(PIPELINE_LABEL)
                    .expect("Sdf mesh must be initialized with the dotrix_voxel startup system"),
            )
            .unwrap();

        let grid_size = grid.total_size();
        let scale = Mat4::from_nonuniform_scale(grid_size[0], grid_size[1], grid_size[2]);
        let uniform = SdfBufferData {
            cube_transform: scale.into(),
            inv_cube_transform: scale.invert().unwrap().into(),
            world_transform: Mat4::identity().into(),
            inv_world_transform: Mat4::identity().into(),
            grid_dimensions: grid_size,
            padding: Default::default(),
        };
        renderer.load_buffer(&mut sdf.data, bytemuck::cast_slice(&[uniform]));

        if !sdf.pipeline.ready(&renderer) {
            let lights_buffer = globals
                .get::<LightStorageBuffer>()
                .expect("Light buffer must be loaded");

            let sampler = globals.get::<Sampler>().expect("Sampler must be loaded");

            if let Some(shader) = assets.get(sdf.pipeline.shader) {
                renderer.bind(
                    &mut sdf.pipeline,
                    PipelineLayout::Render {
                        label: String::from(PIPELINE_LABEL),
                        mesh,
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
                                    Binding::Uniform("Data", Stage::All, &sdf.data),
                                    Binding::Texture3D("Sdf", Stage::All, &sdf.buffer),
                                ],
                            ),
                        ],
                        options: RenderOptions::default(),
                    },
                );
            }
        }

        renderer.draw(&mut sdf.pipeline, mesh, &Default::default());
    }
}

pub(super) fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(render));
    camera::extension(app);
    lights::extension(app);
}
