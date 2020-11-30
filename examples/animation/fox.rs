// The Fox model used in this example is taken from Khronos repository
// https://github.com/KhronosGroup/glTF-Sample-Models
// under following License:
// CC0: Low poly fox by PixelMannen
// CC-BY 4.0: Rigging and animation by @tomkranis on Sketchfab
// glTF conversion by @AsoboStudio and @scurest
//

use dotrix::{
    Dotrix,
    assets::{Animation, Mesh, Skin, Texture},
    components::{Animator, Light, SkeletalModel},
    ecs::{Mut, RunLevel, System},
    services::{Assets, Camera, World},
    systems::{skeletal_renderer, skeletal_animation},
};

fn main() {

    Dotrix::application("Fox Skeletal Animation Example")
        .with_system(System::from(skeletal_renderer).with(RunLevel::Render))
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(skeletal_animation))
        .with_service(Assets::new())
        .with_service(Camera::new(200.0, std::f32::consts::PI / 4.0, 100.0))
        .with_service(World::new())
        .run();

}

fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    let texture = assets.register::<Texture>("Fox::fox::texture");
    let mesh = assets.register::<Mesh>("Fox::fox::mesh");
    let skin = assets.register::<Skin>("Fox::fox::skin");
    let moves = assets.register::<Animation>("Fox::Walk");
    // let moves = assets.register::<Animation>("Fox::Run");
    // let moves = assets.register::<Animation>("Fox::Survey");

    assets.import("assets/Fox.gltf", "fox");

    world.spawn(Some(
        (SkeletalModel::new(mesh, texture, skin), Animator::looped(moves)),
    ));

    world.spawn(Some((Light::white([200.0, 50.0, 200.0]),)));
}

