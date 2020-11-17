// The Fox model used in this example is taken from Khronos repository
// https://github.com/KhronosGroup/glTF-Sample-Models
// under following License:
// CC0: Low poly fox by PixelMannen
// CC-BY 4.0: Rigging and animation by @tomkranis on Sketchfab
// glTF conversion by @AsoboStudio and @scurest
//

use dotrix::{
    Dotrix,
    assets::{ Mesh, Texture },
    components::{ Light, StaticModel },
    ecs::{ Mut, RunLevel, System },
    services::{ Assets, Camera, World },
    systems::{ static_renderer },
};

fn main() {

    Dotrix::application("GLTF Import Example")
        .with_system(System::from(static_renderer).with(RunLevel::Render))
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(fly_around))
        .with_service(Assets::new())
        .with_service(Camera::new(200.0, std::f32::consts::PI / 2.0, 100.0))
        .with_service(World::new())
        .run();

}

fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    let texture = assets.register::<Texture>("Fox::fox::texture");
    let mesh = assets.register::<Mesh>("Fox::fox::mesh");

    assets.import("assets/Fox.gltf", "fox");

    world.spawn(Some(
        (StaticModel::new(mesh, texture),),
    ));

    world.spawn(Some((Light::white([200.0, 100.0, 200.0]),)));
}

fn fly_around(mut camera: Mut<Camera>) {
    let target = cgmath::Point3::new(0.0, 50.0, 0.0);
    let distance = camera.distance();
    let angle = camera.angle() + 0.002;
    let height = camera.height();

    camera.set(target, distance, angle, height);
}
