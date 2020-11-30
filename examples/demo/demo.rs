use dotrix::{
    Dotrix,
    assets::{ Mesh, Texture },
    components::{ Light, StaticModel },
    ecs::{ Mut, Const, RunLevel, System },
    services::{ Assets, Camera, Frame, World },
    systems::{ static_renderer },
};

fn main() {

    Dotrix::application("Demo Example")
        .with_system(System::from(static_renderer).with(RunLevel::Render))
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(fly_around))
        .with_service(Assets::new())
        .with_service(Camera::new(10.0, std::f32::consts::PI / 2.0, 4.0))
        .with_service(World::new())
        .run();

}

fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    assets.import("assets/crate.png", "crate");

    let texture = assets.register::<Texture>("crate");
    let cube1 = assets.store::<Mesh>(Mesh::cube(), "cube1");
    let cube2 = assets.store::<Mesh>(Mesh::cube2(), "cube2");

    world.spawn(vec![
        (StaticModel::new(cube2, texture),),
        (StaticModel::new(cube1, texture),),
    ]);

    world.spawn(Some((Light::white([10.0, 5.0, 4.0]),)));
}

fn fly_around(frame: Const<Frame>, mut camera: Mut<Camera>) {
    let speed = std::f32::consts::PI / 3.0;
    let target = cgmath::Point3::new(0.0, 0.0, 0.0);
    let distance = camera.distance();
    let angle = camera.angle() + speed * frame.delta().as_secs_f32();
    let height = camera.height();

    camera.set(target, distance, angle, height);
}
