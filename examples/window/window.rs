mod camera;
mod match_finder;
mod settings;
mod skybox;
mod terrain;

use camera::{ CameraMemory, camera_update };
use dotrix::{
    Dotrix,
    ecs::{ Mut, RunLevel, System },
    input::{ ActionMapper, Button, Mapper },
    math::{ Vec3 },
    services::{ Assets, Camera, Frame, Input, World },
    renderer::{ SimpleLight },
    systems::{ overlay_update, world_renderer },
};
use match_finder::MatchFinder;
use settings::Settings;

fn main() {
    let mapper: Mapper<Action> = Mapper::new();

    Dotrix::application("Window Example")
        .with_system(System::from(overlay_update))
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        .with_system(System::from(camera_update))
        .with_system(System::from(match_finder::update))
        .with_system(System::from(settings::startup).with(RunLevel::Startup))
        .with_system(System::from(settings::ui))
        .with_system(System::from(skybox::startup).with(RunLevel::Startup))
        .with_system(System::from(spawn_light).with(RunLevel::Startup))
        .with_system(System::from(terrain::init).with(RunLevel::Startup))
        .with_service(Assets::new())
        .with_service(
            Camera {
                distance: 1.0,
                xz_angle: 0.0,
                ..Default::default()
            }
        )
        .with_service(CameraMemory::new())
        .with_service(Frame::new())
        .with_service(MatchFinder::new())
        .with_service(Settings::default())
        .with_service(Input::new(Box::new(mapper)))
        .with_service(World::new())
        .run();

}

fn spawn_light(mut world: Mut<World>) {
    world.spawn(Some(
        (SimpleLight {
            position: Vec3 { x: 0.0, y: 50.0, z: 0.0 },
            ..Default::default()
        },),
    ));
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
