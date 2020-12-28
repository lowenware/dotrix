// The Fox model used in this example is taken from Khronos repository
// https://github.com/KhronosGroup/glTF-Sample-Models
// under following License:
// CC0: Low poly fox by PixelMannen
// CC-BY 4.0: Rigging and animation by @tomkranis on Sketchfab
// glTF conversion by @AsoboStudio and @scurest
//

use dotrix::{
    Dotrix,
    assets::{ Animation, Mesh, Skin, Texture },
    components::{ Animator, Light, Model },
    ecs::{ Mut, RunLevel, System },
    math::Transform,
    services::{ Assets, Camera, World },
    systems::{ skeletal_animation, world_renderer },
};

fn main() {

    Dotrix::application("Fox Skeletal Animation Example")
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(skeletal_animation))
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        .with_service(Assets::new())
        .with_service(Camera {
            distance: 200.0,
            y_angle: 1.57,
            xz_angle: 0.2,
            ..Default::default()
        })
        .with_service(World::new())
        .run();

}

fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    let texture = assets.register::<Texture>("Fox::fox::texture");
    let mesh = assets.register::<Mesh>("Fox::fox::mesh");
    let skin = assets.register::<Skin>("Fox::fox::skin");
    let walk = assets.register::<Animation>("Fox::Walk");
    let run = assets.register::<Animation>("Fox::Run");
    let survey = assets.register::<Animation>("Fox::Survey");
    let trans1 = Transform {
        translate: cgmath::Vector3::new(-100.0, 0.0, 0.0),
        scale: cgmath::Vector3::new(0.8, 0.8, 0.8),
        ..Default::default()
    };
    let trans2 = Transform {
        scale: cgmath::Vector3::new(0.8, 0.8, 0.8),
        ..Default::default()
    };
    let trans3 = Transform {
        translate: cgmath::Vector3::new(100.0, 0.0, 0.0),
        scale: cgmath::Vector3::new(0.8, 0.8, 0.8),
        ..Default::default()
    };

    assets.import("examples/animation/Fox.gltf");

    world.spawn(vec![
        (
            Model {
                mesh,
                texture,
                skin,
                transform: trans1,
                ..Default::default()
            },
            Animator::looped(walk)
        ), (
            Model {
                mesh,
                texture,
                skin,
                transform: trans2,
                ..Default::default()
            },
            Animator::looped(survey)
        ), (
            Model {
                mesh,
                texture,
                skin,
                transform: trans3,
                ..Default::default()
            },
            Animator::looped(run)
        )
    ]);

    world.spawn(Some((Light::white([200.0, 50.0, 200.0]),)));
}

