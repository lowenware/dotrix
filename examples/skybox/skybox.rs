use dotrix::{
    Dotrix,
    assets::{Texture},
    components::{Light, SkyBox},
    ecs::{Mut, Const, RunLevel, System},
    services::{Assets, Camera, Frame, World},
    systems::{skybox_renderer},
};

fn main() {

    Dotrix::application("SkyBox Example")
        .with_system(System::from(skybox_renderer).with(RunLevel::Render))
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(fly_around))
        .with_service(Assets::new())
        .with_service(Camera::new(2.6, std::f32::consts::PI / 2.0, 0.5))
        .with_service(World::new())
        .run();

}

fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    let primary_texture = [
        assets.register::<Texture>("skybox_right"),
        assets.register::<Texture>("skybox_left"),
        assets.register::<Texture>("skybox_top"),
        assets.register::<Texture>("skybox_bottom"),
        assets.register::<Texture>("skybox_back"),
        assets.register::<Texture>("skybox_front"),
    ];

    // The skybox cubemap was downloaded from https://opengameart.org/content/elyvisions-skyboxes
    // These files were licensed as CC-BY 3.0 Unported on 2012/11/7
    assets.import("assets/skybox/skybox_right.png", "skybox_right");
    assets.import("assets/skybox/skybox_left.png", "skybox_left");
    assets.import("assets/skybox/skybox_top.png", "skybox_top");
    assets.import("assets/skybox/skybox_bottom.png", "skybox_bottom");
    assets.import("assets/skybox/skybox_front.png", "skybox_front");
    assets.import("assets/skybox/skybox_back.png", "skybox_back");

    world.spawn(vec![
        (SkyBox { primary_texture, ..Default::default() },),
    ]);

    world.spawn(Some((Light::white([10.0, 5.0, 4.0]),)));
}

fn fly_around(frame: Const<Frame>, mut camera: Mut<Camera>) {
    let speed = std::f32::consts::PI / 8.0;
    let target = cgmath::Point3::new(0.0, 0.6, 0.0);
    let distance = camera.distance();
    let angle = camera.angle() + speed * frame.delta().as_secs_f32();
    let height = camera.height();

    camera.set(target, distance, angle, height);
}
