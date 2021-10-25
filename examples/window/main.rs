mod camera;
mod match_finder;
mod settings;

use camera::{ camera_update };
use dotrix::prelude::*;
use dotrix::{ Camera };
use dotrix::{ egui, overlay };

use match_finder::MatchFinder;
use settings::Settings;

fn main() {
    Dotrix::application("Dotrix: Window Example")
        .with(System::from(startup))
        .with(System::from(settings::startup))

        .with(System::from(settings::ui))

        .with(System::from(camera_update))
        .with(System::from(match_finder::update))

        .with(Service::from(MatchFinder::new()))
        .with(Service::from(Settings::default()))

        .with(overlay::extension)
        .with(egui::extension)
        .run();

}

fn startup(mut camera: Mut<Camera>) {
    camera.distance = 1.0;
    camera.xz_angle = 0.0;
}
