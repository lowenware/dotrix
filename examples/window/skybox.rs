use dotrix::{
    assets::{ Texture },
    components::{ SkyBox },
    ecs::{ Mut },
    services::{ Assets, World },
};

pub fn startup(mut world: Mut<World>, mut assets: Mut<Assets>) {
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
    assets.import("examples/window/assets/skybox_right.png");
    assets.import("examples/window/assets/skybox_left.png");
    assets.import("examples/window/assets/skybox_top.png");
    assets.import("examples/window/assets/skybox_bottom.png");
    assets.import("examples/window/assets/skybox_front.png");
    assets.import("examples/window/assets/skybox_back.png");

    world.spawn(vec![
        (SkyBox { primary_texture, ..Default::default() },),
    ]);
}
