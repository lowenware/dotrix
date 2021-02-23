use dotrix::{
    ecs::Mut,
    renderer::SkyBox,
    services::{ Assets, World }
};

pub fn init(mut world: Mut<World>, mut assets: Mut<Assets>) {
    // Import skybox textures
    assets.import("examples/light/assets/skybox_right.png");
    assets.import("examples/light/assets/skybox_left.png");
    assets.import("examples/light/assets/skybox_top.png");
    assets.import("examples/light/assets/skybox_bottom.png");
    assets.import("examples/light/assets/skybox_back.png");
    assets.import("examples/light/assets/skybox_front.png");

    // Get slice with textures
    let primary_texture = [
        assets.register("skybox_right"),
        assets.register("skybox_left"),
        assets.register("skybox_top"),
        assets.register("skybox_bottom"),
        assets.register("skybox_back"),
        assets.register("skybox_front"),
    ];

    // Spawn skybox
    world.spawn(Some(
        (SkyBox { primary_texture, ..Default::default() },),
    ));
}