mod fox;
mod settings;

use dotrix::{
    Dotrix,
    ecs::{ Mut, RunLevel, System },
    input::{ Mapper },
    math::{ Point3, Vec3 },
    renderer::{ AmbientLight, SimpleLight },
    services::{ Assets, Camera, Frame, Input, World },
    systems::{ overlay_update, skeletal_animation, world_renderer },
};
use settings::{ Settings };

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Action {}

fn main() {
    Dotrix::application("egui Example")
        .with_system(System::from(spawn_lights).with(RunLevel::Startup))
        .with_system(System::from(fox::startup).with(RunLevel::Startup))
        .with_system(System::from(skeletal_animation))
        .with_system(System::from(settings::startup).with(RunLevel::Startup))
        .with_system(System::from(overlay_update))
        .with_system(System::from(settings::ui))
        .with_system(System::from(settings::update_camera))
        .with_system(System::from(settings::update_fox))
        .with_system(System::from(settings::update_lights))
        .with_system(System::from(world_renderer).with(RunLevel::Render))
        .with_service(Assets::new())
        .with_service(Frame::new())
        .with_service(Settings::default())
        .with_service(Camera {
            distance: 222.0,
            y_angle: 0.74,
            xz_angle: 0.25,
            target: Point3::new(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .with_service(World::new())
        .with_service(Input::new(Box::new(Mapper::<Action>::new())))
        .run();
}

pub fn spawn_lights(mut world: Mut<World>) {
    world.spawn(Some((AmbientLight {
        ..Default::default()
    },)));

    world.spawn(Some((SimpleLight {
        position: Vec3::new(0.0, 500.0, 0.0),
        ..Default::default()
    },)));

    world.spawn(Some((SimpleLight {
        position: Vec3::new(200.0, 50.0, 200.0),
        ..Default::default()
    },)));
}