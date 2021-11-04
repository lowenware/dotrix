use dotrix_core::{
    Application,
    Assets,
    Color,
    Globals,
    Id,
    Pipeline,
    Pose,
    Renderer,
    Transform,
    World,
};
use dotrix_core::assets::{ Mesh, Shader, Skin, Texture };
use dotrix_core::ecs::{ Mut, Const, System };
use dotrix_core::camera::ProjView;
use dotrix_core::renderer::{
    BindGroup,
    Binding,
    PipelineLayout,
    PipelineOptions,
    Sampler,
    Stage,
};


use dotrix_math::{ Vec3, Quat, Rad, Rotation3 };

use crate::{ Lights, Material, Model };

pub const PIPELINE_LABEL: &str = "pbr::skeletal";

pub struct Entity {
    /// Mesh asset ID
    pub mesh: Id<Mesh>,
    /// Skin asset ID
    pub skin: Id<Skin>,
    /// Texture asset ID
    pub texture: Id<Texture>,
    /// Albedo color
    pub albedo: Color,
    /// Shader asset ID
    pub shader: Id<Shader>,
    /// Translation vector
    pub translate: Vec3,
    /// Rotation quaternion
    pub rotate: Quat,
    /// Scale vector
    pub scale: Vec3,
}

impl Entity {
    pub fn some(self) -> Option<(Model, Pose, Material, Transform, Pipeline)> {
        Some((
            Model::from(self.mesh),
            Pose::from(self.skin),
            Material {
                albedo: self.albedo,
                texture: self.texture,
                ..Default::default()
            },
            Transform {
                translate: self.translate,
                rotate: self.rotate,
                scale: self.scale,
            },
            Pipeline {
                shader: self.shader,
                ..Default::default()
            },
        ))
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            mesh: Id::default(),
            skin: Id::default(),
            texture: Id::default(),
            albedo: Color::default(),
            shader: Id::default(),
            translate: Vec3::new(0.0, 0.0, 0.0),
            rotate: Quat::from_angle_y(Rad(0.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

pub fn render(
    mut renderer: Mut<Renderer>,
    mut assets: Mut<Assets>,
    globals: Const<Globals>,
    world: Const<World>,
) {
    let query = world.query::<(
        &mut Model,
        &mut Pose,
        &mut Material,
        &mut Transform,
        &mut Pipeline
    )>();

    for (model, pose, material, transform, pipeline) in query {

        if pipeline.shader.is_null() {
            pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL)
                .unwrap_or_default();
        }

        // check if model is disabled or already rendered
        if !pipeline.cycle(&renderer) { continue; }

        // Skip rendering if some data was not loaded yet
        if !model.load(&renderer, &mut assets) { continue; }

        if !material.load(&renderer, &mut assets) { continue; }

        if !pose.load(&renderer, &assets) { continue; }

        model.transform(&renderer, transform);

        let mesh = assets.get(model.mesh).unwrap();

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

                renderer.bind(pipeline, PipelineLayout {
                    label: String::from(PIPELINE_LABEL),
                    mesh,
                    shader,
                    bindings: &[
                        BindGroup::new("Globals", vec![
                            Binding::Uniform("ProjView", Stage::Vertex, &proj_view.uniform),
                            Binding::Sampler("Sampler", Stage::Fragment, sampler),
                            Binding::Uniform("Lights", Stage::Fragment, &lights.uniform),
                        ]),
                        BindGroup::new("Locals", vec![
                            Binding::Uniform("Transform", Stage::Vertex, &model.transform),
                            Binding::Uniform("Material", Stage::Fragment, &material.uniform),
                            Binding::Texture("Texture", Stage::Fragment, &texture.buffer),
                            Binding::Uniform("Joints", Stage::Vertex, &pose.uniform),
                        ])
                    ],
                    options: PipelineOptions::default()
                });
            }
        }

        renderer.run(pipeline, mesh);
    }
}

pub fn startup(mut assets: Mut<Assets>) {
    let shader = include_str!("shaders/skeletal.wgsl");
    assets.store_as(
        Shader {
            name: String::from(PIPELINE_LABEL),
            code: Lights::add_to_shader(shader, 0, 2),
            ..Default::default()
        },
        PIPELINE_LABEL
    );
}

pub fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(render));
}

