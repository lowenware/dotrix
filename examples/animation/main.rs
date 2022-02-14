mod fox;
mod settings;

use dotrix::math::Vec3;
use dotrix::pbr::{self, Light};
use dotrix::prelude::*;
use dotrix::{egui, overlay};
use dotrix::{Camera, Color, System, World};

use settings::Settings;

fn main() {
    Dotrix::application("Dotrix: Animation Example")
        .with(System::from(startup))
        .with(System::from(fox::startup))
        .with(System::from(settings::ui))
        .with(System::from(settings::update_camera))
        .with(System::from(settings::update_fox))
        .with(System::from(settings::update_lights))
        .with(Service::from(Settings::default()))
        .with(overlay::extension)
        .with(egui::extension)
        .with(pbr::extension)
        .run();
}

pub fn startup(mut camera: Mut<Camera>, mut world: Mut<World>) {
    camera.distance = 222.0;
    camera.pan = 0.74;
    camera.tilt = 0.25;
    camera.target = Vec3::new(0.0, 0.5, 0.0);

    world.spawn(vec![
        (Light::ambient(),),
        (Light::Simple {
            position: Vec3::new(0.0, 500.0, 0.0),
            color: Color::white(),
            intensity: 0.8,
            enabled: true,
        },),
        (Light::Simple {
            position: Vec3::new(200.0, 50.0, 200.0),
            color: Color::white(),
            intensity: 0.8,
            enabled: true,
        },),
    ]);
}
