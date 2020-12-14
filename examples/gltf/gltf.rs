use dotrix::{
    Dotrix,
    assets::{ Mesh, Texture },
    components::{ Light, Model },
    ecs::{ Const, Mut, RunLevel, System },
    services::{ Assets, Camera, Frame, World },
    systems::world_renderer,
};

fn main() {

    Dotrix::application("GLTF Import Example")
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(fly_around))
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        .with_service(Assets::new())
        .with_service(Camera::new(10.0, std::f32::consts::PI / 2.0, 2.5))
        .with_service(World::new())
        .run();

}

fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    let texture = assets.register::<Texture>("gray");
    let mesh = assets.register::<Mesh>("Female::Cube::mesh");

    assets.import("assets/Female.gltf");
    assets.import("assets/gray.png");

    world.spawn(Some(
        (Model { mesh, texture, ..Default::default() },),
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
