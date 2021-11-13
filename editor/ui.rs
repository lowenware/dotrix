use dotrix::prelude::*;
use dotrix::{ Assets, Color, Frame, Renderer, Window };
use dotrix::egui::{ Egui, native as egui };
use dotrix::overlay::Overlay;

mod terrain;
mod objects;
mod widgets;

pub use widgets::*;
pub use terrain::HeightMapAction;
use crate::brush::Brush;

#[derive(Eq, PartialEq)]
pub enum Mode {
    Viewer,
    Terrain,
    Objects
}

pub struct Controls {
    pub mode: Mode,
    pub terrain: terrain::Controls,
    pub objects: objects::Controls,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            mode: Mode::Viewer,
            terrain: terrain::Controls::default(),
            objects: objects::Controls::default(),
        }
    }
}

pub fn startup(
    mut assets: Mut<Assets>,
    mut controls: Mut<Controls>,
    mut renderer: Mut<Renderer>,
    window: Const<Window>,
) {
    controls.terrain.set_brush_texture(assets.register("dotrix::editor::brush"));
    renderer.set_clear_color(Color::rgb(0.02, 0.02, 0.02));
    window.set_maximized(true);
}

pub fn show(
    mut controls: Mut<Controls>,
    mut brush: Mut<Brush>,
    frame: Const<Frame>,
    overlay: Const<Overlay>
) {
    let egui_overlay = overlay.get::<Egui>()
        .expect("Egui extension must be enabled");

    egui::Area::new("FPS counter")
        .fixed_pos(egui::pos2(16.0, 32.0))
        .show(&egui_overlay.ctx, |ui| {
            ui.colored_label(
                egui::Rgba::from_rgb(255.0, 255.0, 255.0),
                format!("FPS: {:.1}", frame.fps())
            );
        });

    egui::TopBottomPanel::top("menu_bar").show(&egui_overlay.ctx, |ui| {
        ui.horizontal(|ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "🔷 Dotrix", |ui| {
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory().reset_areas();
                    }
                });

            ui.separator();

            ui.selectable_value(&mut controls.mode, Mode::Viewer, "Viewer");
            ui.selectable_value(&mut controls.mode, Mode::Terrain, "Terrain");
            ui.selectable_value(&mut controls.mode, Mode::Objects, "Objects");
            });
        });
    });

    if controls.mode != Mode::Viewer {
        egui::SidePanel::right("Sidebar")
            .min_width(300.0)
            .resizable(false)
            .show(&egui_overlay.ctx, |ui| {

            match controls.mode {
                Mode::Terrain => terrain::show(
                    &egui_overlay.ctx,
                    ui,
                    &mut controls.terrain,
                    &mut brush
                ),
                Mode::Objects => objects::show(ui, &mut controls.objects),
                _ => {}
            };
        });
    }
}

