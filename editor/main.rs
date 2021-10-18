use dotrix::prelude::*;
use dotrix::{ egui, overlay, ray, pbr };

mod brush;
mod scene;
mod terrain;
mod ui;

fn main() {
    Dotrix::application("Mythstic World Editor")
        .with(egui::extension)
        .with(overlay::extension)
        .with(pbr::extension)
        .with(ray::extension)
        .with(terrain::extension)

        .with(System::from(brush::startup))
        .with(System::from(ui::startup))
        .with(System::from(terrain::startup))
        .with(System::from(terrain::load))
        .with(System::from(scene::startup))

        .with(System::from(brush::update))
        .with(System::from(scene::control))
        .with(System::from(ui::show))

        .with(Service::from(brush::Brush::default()))
        .with(Service::from(ui::Controls::default()))

        .run();
}
