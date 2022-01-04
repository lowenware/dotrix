use dotrix::camera;
use dotrix::prelude::*;
use dotrix::sky::{skybox, SkyBox};
use dotrix::{Assets, Camera, CubeMap, Pipeline, World};

fn main() {
    Dotrix::application("Dotrix: SkyBox Example")
        .with(System::from(startup))
        .with(System::from(camera::control))
        .with(skybox::extension)
        .run();
}

fn startup(mut camera: Mut<Camera>, mut world: Mut<World>, mut assets: Mut<Assets>) {
    camera.distance = 1.0;
    camera.xz_angle = 0.0;

    // Import skybox textures
    assets.import("assets/skybox-day/skybox_right.png");
    assets.import("assets/skybox-day/skybox_left.png");
    assets.import("assets/skybox-day/skybox_top.png");
    assets.import("assets/skybox-day/skybox_bottom.png");
    assets.import("assets/skybox-day/skybox_back.png");
    assets.import("assets/skybox-day/skybox_front.png");

    // Spawn skybox
    world.spawn(Some((
        SkyBox {
            view_range: 500.0,
            ..Default::default()
        },
        CubeMap {
            right: assets.register("skybox_right"),
            left: assets.register("skybox_left"),
            top: assets.register("skybox_top"),
            bottom: assets.register("skybox_bottom"),
            back: assets.register("skybox_back"),
            front: assets.register("skybox_front"),
            ..Default::default()
        },
        Pipeline::default(),
    )));
}
