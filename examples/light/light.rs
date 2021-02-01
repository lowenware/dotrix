mod car;
mod settings;
mod skybox;
mod terrain;

use dotrix::{
    Dotrix,
    components:: { AmbientLight, DirLight, PointLight, SimpleLight, SpotLight },
    ecs::{ Mut, RunLevel, System },
    input::{ Mapper },
    math::{ Point3 },
    services::{ Assets, Camera, Frame, Input, World },
    systems::{ overlay_update, world_renderer },
};
use settings::{ Settings };

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Action {}

fn main() {
    Dotrix::application("Light Example")
        .with_system(System::from(car::init).with(RunLevel::Startup))
        .with_system(System::from(settings::init).with(RunLevel::Startup))
        .with_system(System::from(skybox::init).with(RunLevel::Startup))
        .with_system(System::from(terrain::init).with(RunLevel::Startup))
        .with_system(System::from(spawn_lights).with(RunLevel::Startup))
        .with_system(System::from(overlay_update))
        .with_system(System::from(settings::ui))
        .with_system(System::from(settings::update_camera))
        .with_system(System::from(settings::update_settings))
        .with_system(System::from(car::update))
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        .with_service(Assets::new())
        .with_service(Frame::new())
        .with_service(Settings::default())
        .with_service(Camera {
            distance: 6.6,
            y_angle: 2.5,
            xz_angle: 0.275,
            target: Point3::new(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .with_service(World::new())
        .with_service(Input::new(Box::new(Mapper::<Action>::new())))
        .run();
}

/// This will mark light as editable. This exists because we don't want to edit police car lights.
struct Editable;

/// Spawn editable lights. Light properties is set with settings.
fn spawn_lights(mut world: Mut<World>) {
    world.spawn(Some((AmbientLight::default(), Editable {})));
    world.spawn(Some((DirLight::default(), Editable {})));
    world.spawn(Some((PointLight::default(), Editable {})));
    world.spawn(Some((SimpleLight::default(), Editable {})));
    world.spawn(Some((SpotLight::default(), Editable {})));
}