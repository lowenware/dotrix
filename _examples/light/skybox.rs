use dotrix::ecs::Mut;
use dotrix::renderer::Render;
use dotrix::sky::SkyBox;
use dotrix::{Assets, CubeMap, World};

pub fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
    // Import skybox textures
    assets.import("assets/skybox-night/skybox_right.png");
    assets.import("assets/skybox-night/skybox_left.png");
    assets.import("assets/skybox-night/skybox_top.png");
    assets.import("assets/skybox-night/skybox_bottom.png");
    assets.import("assets/skybox-night/skybox_back.png");
    assets.import("assets/skybox-night/skybox_front.png");

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
        Render::default(),
    )));
}
