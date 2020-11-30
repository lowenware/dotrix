use dotrix::{
    Dotrix,
    assets::{Animation, Mesh, Skin, Texture},
    components::{Animator, Light, SkeletalModel},
    ecs::{Const, Mut, RunLevel, System},
    services::{Assets, Camera, Frame, World},
    systems::{skeletal_renderer, skeletal_animation},
};

fn main() {

    Dotrix::application("Female Skeletal Animation Example")
        .with_system(System::from(skeletal_renderer).with(RunLevel::Render))
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(fly_around))
        .with_system(System::from(skeletal_animation))
        .with_service(Assets::new())
        .with_service(Camera::new(7.0, std::f32::consts::PI / 4.0, 2.5))
        .with_service(World::new())
        .run();

}

fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    let mesh = assets.register::<Mesh>("Female::Cube::mesh");
    let skin = assets.register::<Skin>("Female::Cube::skin");
    let moves = assets.register::<Animation>("Female::run");
    let texture = assets.register::<Texture>("gray");

    assets.import("assets/Female.gltf", "female");
    assets.import("assets/gray.png", "gray");

    world.spawn(Some(
        (SkeletalModel::new(mesh, texture, skin), Animator::looped(moves)),
    ));

    world.spawn(Some((Light::white([200.0, 100.0, 200.0]),)));
}

fn fly_around(frame: Const<Frame>, mut camera: Mut<Camera>) {
    let speed = std::f32::consts::PI / 3.0;
    let target = cgmath::Point3::new(0.0, 2.5, 0.0);
    let distance = camera.distance();
    let angle = camera.angle() + speed * frame.delta().as_secs_f32();
    let height = camera.height();

    camera.set(target, distance, angle, height);
}
