use crate::{
    assets::{ Assets, Mesh, Shader, Texture },
    components::{ Lights, Material, Model, Pipeline, Transform },
    ecs::{ Mut, Const },
    camera::{ ProjView },
    generics::{ Id, Color },
    globals::Globals,
    renderer::{ 
        AttributeFormat,
        BindGroup,
        Binding,
        Bindings,
        PipelineLayout,
        Renderer,
        Sampler,
        Stage,
    },
    world:: World,
};

use dotrix_math::{ Vec3, Quat, Rad, Rotation3 };

pub const PIPELINE_LABEL: &str = "pbr::solid";

pub struct Entity {
    pub mesh: Id<Mesh>,
    pub texture: Id<Texture>,
    pub albedo: Color,
    pub shader: Id<Shader>,
    /// Translation vector
    pub translate: Vec3,
    /// Rotation quaternion
    pub rotate: Quat,
    /// Scale vector
    pub scale: Vec3,
}

impl Entity {
    pub fn as_tuple(self) -> Option<((Model, Material, Transform, Pipeline))> {
        Some((
            Model {
                mesh: self.mesh,
                ..Default::default()
            },
            Material {
                albedo: self.albedo,
                texture: self.texture,
                ..Default::default()
            },
            Transform {
                translate: self.translate,
                rotate: self.rotate,
                scale: self.scale,
                ..Default::default()
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
            texture: Id::default(),
            albedo: Color::default(),
            pipeline: Id::default(),
            translate: Vec3::new(0.0, 0.0, 0.0),
            rotate: Quat::from_angle_y(Rad(0.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

/*
pub fn create_pipeline(shader: Id<Shader>) -> Pipeline {
    Pipeline {
        label: String::from(PIPELINE_LABEL),
        shader,
        vertex_attributes: Some(vec![
            AttributeFormat::Float32x3,
            AttributeFormat::Float32x3,
            AttributeFormat::Float32x2,
        ]),
        layout: vec![
            BindGroup::layout("Globals", vec![
                Binding::Uniform(stage::vertex("ProjView")),
                Binding::Sampler(stage::fragment("Texture Sampler")),
                Binding::Uniform(stage::fragment("Lights")),
            ]),
            BindGroup::layout("Locals", vec![
                Binding::Uniform(stage::vertex("Transform")),
                Binding::Uniform(stage::vertex("Material")),
                Binding::Texture(stage::fragment("Texture")),
            ]),
        ],
        use_depth_buffer: true,
        ..Default::default()
    }
}
*/

pub fn render(
    mut renderer: Mut<Renderer>,
    mut assets: Mut<Assets>,
    globals: Const<Globals>,
    world: Const<World>,
) {
    let query = world.query::<(&mut Model, &mut Material, &mut Transform, &mut Pipeline)>();
    for (model, material, transform, pipeline) in query {

        if pipeline.shader.is_none() {
            pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL);
        }

        // check if model is disabled or already rendered
        if !pipeline.cycle(&renderer) { continue; }

        if !model.load(&renderer, &mut assets) { continue; }

        if !material.load(&renderer, &mut assets) { continue; }

        transform.load(&renderer);

        let mesh = assets.get(model.mesh).unwrap();

        if pipeline.not_ready() {
            if let Some(shader) = assets.get(pipeline.shader) {
                if !shader.loaded() { continue; }

                let texture = assets.get(material.texture).unwrap();

                let proj_view = globals.get::<ProjView>()
                    .expect("ProjView buffer must be loaded");

                let sampler = globals.get::<Sampler>()
                    .expect("ProjView buffer must be loaded");

                let lights = globals.get::<Lights>()
                    .expect("Lights buffer must be loaded");

                renderer.bind(&mut pipeline, PipelineLayout {
                    label: String::from(PIPELINE_LABEL),
                    mesh: mesh,
                    shader: shader,
                    bindings: &[
                        BindGroup::entry("Globals", vec![
                            Binding::Uniform("ProjView", Stage::Vertex, &proj_view.buffer),
                            Binding::Sampler("Sampler", Stage::Fragment, sampler),
                            Binding::Uniform("Lights", Stage::Fragment, &lights.buffer),
                        ]),
                        BindGroup::entry("Locals", vec![
                            Binding::Uniform("Transform", Stage::Vertex, &transform.buffer),
                            Binding::Uniform("Material", Stage::Vertex, &material.buffer),
                            Binding::Texture("Texture", Stage::Fragment, &texture.buffer),
                        ])
                    ]
                });
            }
        }

        renderer.run(&pipeline, &mesh);
    }
}
