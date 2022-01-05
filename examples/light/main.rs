mod car;
mod settings;
mod skybox;
mod terrain;

use dotrix::math::Point3;
use dotrix::pbr::Light;
use dotrix::prelude::*;
use dotrix::{egui, overlay, pbr, sky};
use dotrix::{Camera, World};

use settings::Settings;

fn main() {
    Dotrix::application("Dotrix: Light Example")
        // systems
        .with(System::from(car::startup))
        .with(System::from(skybox::startup))
        .with(System::from(terrain::startup))
        .with(System::from(startup))
        .with(System::from(settings::ui))
        .with(System::from(settings::update_camera))
        .with(System::from(settings::update_settings))
        .with(System::from(car::update))
        // Services
        .with(Service::from(Settings::default()))
        // Extensions
        .with(overlay::extension)
        .with(egui::extension)
        .with(pbr::extension)
        .with(sky::skybox::extension)
        // Execute
        .run();
}

/// This will mark light as editable. This exists because we don't want to edit police car lights.
struct Editable;

/// Spawn editable lights. Light properties is set with settings.
fn startup(mut camera: Mut<Camera>, mut world: Mut<World>) {
    camera.distance = 6.6;
    camera.y_angle = 2.5;
    camera.xz_angle = 0.275;
    camera.target = Point3::new(0.0, 0.5, 0.0);

    world.spawn(Some((Light::ambient(), Editable {})));
    world.spawn(Some((Light::directional(), Editable {})));
    world.spawn(Some((Light::point(), Editable {})));
    world.spawn(Some((Light::simple(), Editable {})));
    world.spawn(Some((Light::spot(), Editable {})));
}
