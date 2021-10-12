use dotrix::egui::native as egui;
use dotrix::Id;
use dotrix::assets::Texture;
use dotrix::math::Vec2u;
use dotrix::terrain::{ Component, Noise };

#[derive(Eq, PartialEq)]
pub enum Brush {
    Elevate,
    Flatten,
    Noise
}

pub struct Controls {
    component: Component,
    tiles_per_x: u32,
    tiles_per_z: u32,
    size_locked: bool,
    heightmap_size: u32,
    max_height: f32,
    brush: Brush,
    brush_size: f32,
    brush_elevate: i8,
    brush_flatten: i8,
    noise: Noise,
    brush_texture: Id<Texture>,
}

impl Controls {
    pub fn set_brush_texture(&mut self, brush_texture: Id<Texture>) {
        self.brush_texture = brush_texture;
    }
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            component: Component::Standard,
            tiles_per_x: 1,
            tiles_per_z: 1,
            size_locked: true,
            heightmap_size: 4096,
            max_height: 512.0,
            brush: Brush::Elevate,
            brush_size: 64.0,
            brush_elevate: 1,
            brush_flatten: 1,
            noise: Noise::default(),
            brush_texture: Id::default(),
        }
    }
}


pub fn show(ui: &mut egui::Ui, controls: &mut Controls) {
    egui::CollapsingHeader::new("Brush")
        .default_open(true)
        .show(ui, |ui| show_brush(ui, controls));
    ui.separator();
    egui::CollapsingHeader::new("Height Map")
        .default_open(true)
        .show(ui, |ui| show_heightmap_properties(ui, controls));
    ui.separator();
    egui::CollapsingHeader::new("Mesh")
        .default_open(true)
        .show(ui, |ui| show_mesh_properties(ui, controls));
}


fn show_heightmap_properties(ui: &mut egui::Ui, controls: &mut Controls) {
    super::tool_grid("heightmap_properties").show(ui, |ui| {
        ui.label("Lock size");
        if ui.checkbox(&mut controls.size_locked, "X == Z").changed() {
            if controls.size_locked {
                if controls.tiles_per_x > controls.tiles_per_z {
                    controls.tiles_per_z = controls.tiles_per_x;
                } else {
                    controls.tiles_per_x = controls.tiles_per_z;
                }
            }
        }
        ui.end_row();

        let quads_per_side = controls.component.units_per_side() as u32;
        let mut size_x = controls.tiles_per_x * quads_per_side + 1;
        let limit_x = size_x;
        let mut size_z = controls.tiles_per_z * quads_per_side + 1;

        ui.label("Size by X");
        if ui.add(egui::DragValue::new(&mut size_x)
            .speed(quads_per_side)
            .clamp_range(std::ops::RangeInclusive::new(0, 8129))
        ).changed() {
            controls.tiles_per_x = size_x / quads_per_side;
            if controls.size_locked {
                controls.tiles_per_z = controls.tiles_per_x;
            }
        }
        ui.end_row();

        ui.label("Size by Z");
        if ui.add(egui::DragValue::new(&mut size_z)
            .speed(quads_per_side)
            .clamp_range(std::ops::RangeInclusive::new(0, 8129))
        ).changed() {
            controls.tiles_per_z = size_z / quads_per_side;
            if controls.size_locked {
                controls.tiles_per_x = controls.tiles_per_z;
            }
        }
        ui.end_row();
    });
}


fn show_mesh_properties(ui: &mut egui::Ui, controls: &mut Controls) {
    super::tool_grid("mesh_properties").show(ui, |ui| {

        ui.label("Max Height");
        ui.add(egui::DragValue::new(&mut controls.max_height)
            .clamp_range(std::ops::RangeInclusive::new(32, 8192)));
        ui.end_row();

    });
}

fn show_brush(ui: &mut egui::Ui, controls: &mut Controls) {
    super::tool_grid("brush").show(ui, |ui| {
        ui.label("Mode");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut controls.brush, Brush::Elevate, "Elevate");
            ui.selectable_value(&mut controls.brush, Brush::Flatten, "Flatten");
            ui.selectable_value(&mut controls.brush, Brush::Noise, "Noise");
        });
        ui.end_row();

        ui.label("Size");
        ui.add(egui::Slider::new(&mut controls.brush_size, 1.0..=512.0));
        ui.end_row();

        match controls.brush {
            Brush::Elevate => {
                ui.label("Direction");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut controls.brush_elevate, 1, "Raise");
                    ui.radio_value(&mut controls.brush_elevate, -1, "Lower");
                });
                ui.end_row();
            },
            Brush::Flatten => {
                ui.label("Direction");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut controls.brush_flatten, 1, "Raise");
                    ui.radio_value(&mut controls.brush_flatten, -1, "Lower");
                });
                ui.end_row();
            },
            Brush::Noise => {
                ui.label("Frequency");
                ui.add(egui::Slider::new(&mut controls.noise.frequency, 1.0..=16.0));
                ui.end_row();

                ui.label("Octaves");
                ui.add(egui::Slider::new(&mut controls.noise.octaves, 1..=8));
                ui.end_row();

                ui.label("Persistence");
                ui.add(egui::Slider::new(&mut controls.noise.persistence, 1.0..=16.0));
                ui.end_row();

                ui.label("Lacunarity");
                ui.add(egui::Slider::new(&mut controls.noise.lacunarity, 1.0..=16.0));
                ui.end_row();

                ui.label("Scale");
                ui.add(egui::Slider::new(&mut controls.noise.scale, 0.01..=8192.0));
                ui.end_row();

                ui.label("Offset");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut controls.noise.offset[0]).prefix("X: "));
                    ui.add(egui::DragValue::new(&mut controls.noise.offset[1]).prefix("Z: "));
                });
                ui.end_row();

                ui.label("Seed");
                ui.add(egui::DragValue::new(&mut controls.noise.seed));
                ui.end_row();
            },
        };

        ui.label("Preview");
        ui.image(egui::TextureId::User(controls.brush_texture.id), [100.0, 100.0]);
        ui.end_row();
    });
}


