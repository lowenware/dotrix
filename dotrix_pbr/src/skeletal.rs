use dotrix_core::assets::{Mesh, Shader, Skin, Texture};
use dotrix_core::camera::ProjView;
use dotrix_core::ecs::{Const, Mut, System};
use dotrix_core::renderer::{
    BindGroup, Binding, DrawArgs, Pipeline, PipelineLayout, Render, RenderOptions, Sampler, Stage,
};
use dotrix_core::{Application, Assets, Color, Globals, Id, Pose, Renderer, Transform, World};

use dotrix_math::{Quat, Rad, Rotation3, Vec3};

use crate::{add_pbr_to_shader, Lights, Material, Model};

pub const PIPELINE_LABEL: &str = "pbr::skeletal";

pub struct Entity {
    /// Mesh asset ID
    pub mesh: Id<Mesh>,
    /// Skin asset ID
    pub skin: Id<Skin>,
    /// Albedo texture asset ID
    pub texture: Id<Texture>,
    /// Albedo color
    pub albedo: Color,
    /// Roughness texture asset ID
    pub roughness_texture: Id<Texture>,
    /// Roughness (Random scatter)
    pub roughness: f32,
    /// Metallic texture asset ID
    pub metallic_texture: Id<Texture>,
    /// Metallic (reflectance)
    pub metallic: f32,
    /// Ambient occulsion texture asset ID
    pub ao_texture: Id<Texture>,
    /// Ambient occulsion
    pub ao: f32,
    /// Id of a normal map asset
    pub normal_texture: Id<Texture>,
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
    pub fn some(self) -> Option<(Model, Pose, Material, Transform, Render)> {
        Some((
            Model::from(self.mesh),
            Pose::from(self.skin),
            Material {
                albedo: self.albedo,
                texture: self.texture,
                roughness: self.roughness,
                metallic: self.metallic,
                ao: self.ao,
                roughness_texture: self.roughness_texture,
                metallic_texture: self.metallic_texture,
                ao_texture: self.ao_texture,
                normal_texture: self.normal_texture,
                ..Default::default()
            },
            Transform {
                translate: self.translate,
                rotate: self.rotate,
                scale: self.scale,
            },
            Pipeline::render(),
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
            roughness: 1.,
            roughness_texture: Id::default(),
            metallic: 0.,
            metallic_texture: Id::default(),
            ao: 0.5,
            ao_texture: Id::default(),
            normal_texture: Id::default(),
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
        &mut Render,
    )>();

    for (model, pose, material, transform, render) in query {
        // check if model is disabled or already rendered
        if !render.pipeline.cycle(&renderer) {
            continue;
        }

        // Skip rendering if some data was not loaded yet
        if !model.load(&renderer, &mut assets) {
            continue;
        }

        if !material.load(&renderer, &mut assets) {
            continue;
        }

        if !pose.load(&renderer, &assets) {
            continue;
        }

        model.transform(&renderer, transform);

        let mesh = assets.get(model.mesh).unwrap();

        if !render.pipeline.ready(&renderer) {
            let shader_id = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
            if let Some(shader) = assets.get(shader_id) {
                if !shader.loaded() {
                    continue;
                }

                let texture = assets.get(material.texture).unwrap();
                let roughness_texture = assets.get(material.roughness_texture).unwrap();
                let metallic_texture = assets.get(material.metallic_texture).unwrap();
                let ao_texture = assets.get(material.ao_texture).unwrap();
                let normal_texture = assets.get(material.normal_texture).unwrap();

                let proj_view = globals
                    .get::<ProjView>()
                    .expect("ProjView buffer must be loaded");

                let sampler = globals
                    .get::<Sampler>()
                    .expect("ProjView buffer must be loaded");

                let lights = globals
                    .get::<Lights>()
                    .expect("Lights buffer must be loaded");

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
                                    Binding::Uniform("ProjView", Stage::Vertex, &proj_view.uniform),
                                    Binding::Sampler("Sampler", Stage::Fragment, sampler),
                                    Binding::Uniform("Lights", Stage::Fragment, &lights.uniform),
                                ],
                            ),
                            BindGroup::new(
                                "Locals",
                                vec![
                                    Binding::Uniform("Transform", Stage::Vertex, &model.transform),
                                    Binding::Uniform(
                                        "Material",
                                        Stage::Fragment,
                                        &material.uniform,
                                    ),
                                    Binding::Texture("Texture", Stage::Fragment, &texture.buffer),
                                    Binding::Texture(
                                        "RoughnessTexture",
                                        Stage::Fragment,
                                        &roughness_texture.buffer,
                                    ),
                                    Binding::Texture(
                                        "MetallicTexture",
                                        Stage::Fragment,
                                        &metallic_texture.buffer,
                                    ),
                                    Binding::Texture(
                                        "AoTexture",
                                        Stage::Fragment,
                                        &ao_texture.buffer,
                                    ),
                                    Binding::Texture(
                                        "NormalTexture",
                                        Stage::Fragment,
                                        &normal_texture.buffer,
                                    ),
                                    Binding::Uniform("Joints", Stage::Vertex, &pose.uniform),
                                ],
                            ),
                        ],
                        options: RenderOptions::default(),
                    },
                );
            }
        }

        renderer.draw(&mut render.pipeline, mesh, &DrawArgs::default());
    }
}

pub fn startup(mut assets: Mut<Assets>) {
    let shader = include_str!("shaders/skeletal.wgsl");
    assets.store_as(
        Shader {
            name: String::from(PIPELINE_LABEL),
            code: add_pbr_to_shader(shader, 0, 2),
            ..Default::default()
        },
        PIPELINE_LABEL,
    );
}

pub fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(render));
}
