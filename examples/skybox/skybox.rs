use dotrix::{
    Dotrix,
    assets::Texture,
    components::SkyBox,
    ecs::{ Mut, RunLevel, System },
    input::{ ActionMapper, Button, Mapper },
    services::{ Assets, Camera, Input, World },
    systems::{ camera_control, world_renderer },
};

fn main() {
    let mapper: Mapper<Action> = Mapper::new();

    Dotrix::application("SkyBox Example")
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        .with_system(System::from(startup).with(RunLevel::Startup))
        .with_system(System::from(camera_control))
        .with_service(Assets::new())
        .with_service(
            Camera {
                distance: 1.0,
                xz_angle: 0.0,
                ..Default::default()
            }
        )
        .with_service(World::new())
        .with_service(Input::new(Box::new(mapper)))
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
    assets.import("examples/skybox/skybox_right.png");
    assets.import("examples/skybox/skybox_left.png");
    assets.import("examples/skybox/skybox_top.png");
    assets.import("examples/skybox/skybox_bottom.png");
    assets.import("examples/skybox/skybox_front.png");
    assets.import("examples/skybox/skybox_back.png");

    world.spawn(vec![
        (SkyBox { primary_texture, ..Default::default() },),
    ]);
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
/// All bindable actions
struct Action;

impl ActionMapper<Action> for Input {
    fn action_mapped(&self, action: Action) -> Option<&Button> {
        let mapper = self.mapper::<Mapper<Action>>();
        mapper.get_button(action)
    }
}
