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
use tera::{Context, Tera};

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
    pub voxel_dimensions: [f32; 4],
    // Dimensions of the voxel grid
    pub grid_dimensions: [f32; 4],
    // pub padding: [f32; 2],
}

unsafe impl bytemuck::Zeroable for SdfBufferData {}
unsafe impl bytemuck::Pod for SdfBufferData {}

pub fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let mut templates = Tera::default();
    templates
        .add_raw_templates(vec![
            ("render", include_str!("./circle_trace/render.wgsl")),
            (
                "accelerated_raytrace",
                include_str!("./circle_trace/accelerated_raytrace.inc.wgsl"),
            ),
            (
                "hemisphere_ambient_occulsion",
                include_str!("./circle_trace/hemisphere_ambient_occulsion.inc.wgsl"),
            ),
            ("lighting", include_str!("./circle_trace/lighting.inc.wgsl")),
            (
                "soft_shadows_closet_approach",
                include_str!("./circle_trace/soft_shadows_closet_approach.inc.wgsl"),
            ),
        ])
        .unwrap();

    let mut context = Context::new();
    // Could select different algorithms here
    let raytrace_algo = templates.render("accelerated_raytrace", &context).unwrap();
    let ao_algo = templates
        .render("hemisphere_ambient_occulsion", &context)
        .unwrap();
    let shadow_algo = templates
        .render("soft_shadows_closet_approach", &context)
        .unwrap();
    let lighting_algo = templates.render("lighting", &context).unwrap();

    context.insert("RAYTRACE_ALGO", &raytrace_algo);
    context.insert("AO_ALGO", &ao_algo);
    context.insert("SHADOWS_ALGO", &shadow_algo);
    context.insert("LIGHTING_ALGO", &lighting_algo);

    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: templates.render("render", &context).unwrap(),
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
        let voxel_size = grid.get_voxel_dimensions();
        let scale = Mat4::from_nonuniform_scale(grid_size[0], grid_size[1], grid_size[2]);
        let uniform = SdfBufferData {
            cube_transform: scale.into(),
            inv_cube_transform: scale.invert().unwrap().into(),
            world_transform: Mat4::identity().into(),
            inv_world_transform: Mat4::identity().into(),
            voxel_dimensions: [voxel_size[0], voxel_size[1], voxel_size[2], 1.],
            grid_dimensions: [grid_size[0], grid_size[1], grid_size[2], 1.],
            // padding: Default::default(),
        };
        // println!("grid_dimensions: {:?}", uniform.grid_dimensions);
        // println!("cube_transform: {:?}", uniform.cube_transform);
        // println!("inv_cube_transform: {:?}", uniform.inv_cube_transform);
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
