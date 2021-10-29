use dotrix::egui::native as egui;
use dotrix::Id;
use dotrix::assets::Texture;
use dotrix::math::Vec2u;
use dotrix::terrain::{ Component, Noise };
use crate::brush::{
    Brush,
    INTENSITY as BRUSH_INTENSITY,
    SIZE as BRUSH_SIZE,
    Mode as BrushMode,
    Shape as BrushShape,
};

pub struct Controls {
    /// Terrain Component Type
    pub component: Component,
    /// Number of components per X
    pub tiles_per_x: u32,
    /// Number of components per Z
    pub tiles_per_z: u32,
    /// If locked, the terrain has square shape
    pub size_locked: bool,
    /// Height of the World
    pub y_scale: f32,
    /// Brush can be elevating or flattening
    pub brush_mode: BrushMode,
    /// Circle or noise
    pub brush_shape: BrushShape,
    /// intensity -1..1
    pub brush_intensity: f32,
    /// Size of the brush
    pub brush_size: u32,
    /// Noise configuration
    pub noise: Noise,
    /// Id of the brush texture
    pub brush_texture: Id<Texture>,
    /// brush reload flag
    pub brush_reload: bool,
    /// terrain map reload falg
    pub map_reload: bool,
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
            tiles_per_x: 4,
            tiles_per_z: 4,
            size_locked: true,
            y_scale: 512.0,
            brush_mode: BrushMode::Elevate,
            brush_shape: BrushShape::Radial,
            brush_intensity: BRUSH_INTENSITY,
            brush_size: BRUSH_SIZE,
            noise: Noise::default(),
            brush_texture: Id::default(),
            brush_reload: true,
            map_reload: true,
        }
    }
}


pub fn show(ui: &mut egui::Ui, controls: &mut Controls, brush: &mut Brush) {


    egui::CollapsingHeader::new("Properties")
        .default_open(true)
        .show(ui, |ui| show_properties(ui, controls));
    ui.separator();

    egui::CollapsingHeader::new("Brush")
        .default_open(true)
        .show(ui, |ui| show_brush(ui, controls, brush));
    ui.separator();

}


fn show_properties(ui: &mut egui::Ui, controls: &mut Controls) {
    super::tool_grid("heightmap_properties").show(ui, |ui| {
        ui.label("Height Map");
        ui.horizontal(|ui| {
            if ui.button("New").clicked() {
                println!("New Height Map");
            }
            if ui.button("Import").clicked() {
                println!("Import Height Map");
            }
            if ui.button("Export").clicked() {
                println!("Export Height Map");
            }
        });
        ui.end_row();

        let quads_per_side = controls.component.units_per_side() as u32;
        let mut size_x = controls.tiles_per_x * quads_per_side + 1;
        let limit_x = size_x;
        let mut size_z = controls.tiles_per_z * quads_per_side + 1;

        ui.label(if controls.size_locked { "Size" } else { "Size by X" });

        ui.horizontal(|ui| {
            if ui.add(egui::DragValue::new(&mut size_x)
                .speed(2 * quads_per_side)
                .clamp_range(std::ops::RangeInclusive::new(63, 8129))
            ).changed() {
                controls.tiles_per_x = size_x / quads_per_side;
                if controls.size_locked {
                    controls.tiles_per_z = controls.tiles_per_x;
                }
                controls.map_reload = true;
            }

            if ui.checkbox(&mut controls.size_locked, "Square").changed() {
                if controls.size_locked {
                    if controls.tiles_per_x > controls.tiles_per_z {
                        controls.tiles_per_z = controls.tiles_per_x;
                    } else {
                        controls.tiles_per_x = controls.tiles_per_z;
                    }
                    controls.map_reload = true;
                }
            }
        });
        ui.end_row();

        if !controls.size_locked {
            ui.label("Size by Z");
            if ui.add(egui::DragValue::new(&mut size_z)
                .speed(quads_per_side)
                .clamp_range(std::ops::RangeInclusive::new(0, 8129))
            ).changed() {
                controls.tiles_per_z = size_z / quads_per_side;
                if controls.size_locked {
                    controls.tiles_per_x = controls.tiles_per_z;
                }
                controls.map_reload = true;
            }
            ui.end_row();
        }

        ui.label("Height Scale");
        if ui.add(
            egui::DragValue::new(&mut controls.y_scale)
                .clamp_range(std::ops::RangeInclusive::new(32, 8192))
        ).changed() {
            controls.map_reload = true;
        }
        ui.end_row();
    });
}


fn show_brush(ui: &mut egui::Ui, controls: &mut Controls, brush: &mut Brush) {
    super::tool_grid("brush").show(ui, |ui| {
        let mut brush_reload = false;
        let mut brush_shape = controls.brush_shape;

        ui.label("Mode");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut controls.brush_mode, BrushMode::Elevate, "Elevate");
            ui.selectable_value(&mut controls.brush_mode, BrushMode::Flatten, "Flatten");
        });
        ui.end_row();

        if brush.mode != controls.brush_mode {
            brush.mode = controls.brush_mode;
            brush_reload = true;
        }

        ui.label("Mode");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut brush_shape, BrushShape::Radial, "Radial");
            ui.selectable_value(&mut brush_shape, BrushShape::Noise, "Noise");
        });
        ui.end_row();

        if brush_shape != controls.brush_shape {
            controls.brush_shape = brush_shape;
            brush_reload = true;
        }

        ui.label("Size");
        brush_reload |= ui.add(
            egui::Slider::new(&mut controls.brush_size, 1..=4096)
        ).drag_released();
        ui.end_row();

        ui.label("Intensity");
        brush_reload |= ui.add(
            egui::Slider::new(&mut controls.brush_intensity, -1.0..=1.0)
        ).drag_released();
        ui.end_row();

        match controls.brush_shape {
            BrushShape::Noise => {
                ui.label("Frequency");
                brush_reload |= ui.add(
                    egui::Slider::new(&mut controls.noise.frequency, 1.0..=16.0)
                ).drag_released();
                ui.end_row();

                ui.label("Octaves");
                brush_reload |= ui.add(
                    egui::Slider::new(&mut controls.noise.octaves, 1..=8)
                ).drag_released();
                ui.end_row();

                ui.label("Persistence");
                brush_reload |= ui.add(
                    egui::Slider::new(&mut controls.noise.persistence, 1.0..=16.0)
                ).drag_released();
                ui.end_row();

                ui.label("Lacunarity");
                brush_reload |= ui.add(
                    egui::Slider::new(&mut controls.noise.lacunarity, 1.0..=16.0)
                ).drag_released();
                ui.end_row();

                ui.label("Scale");
                brush_reload |= ui.add(
                    egui::Slider::new(&mut controls.noise.scale, 0.01..=8192.0)
                ).drag_released();
                ui.end_row();

                ui.label("Offset");
                ui.horizontal(|ui| {
                    brush_reload |= ui.add(
                        egui::DragValue::new(&mut controls.noise.offset[0]).prefix("X: ")
                    ).drag_released();
                    brush_reload |= ui.add(
                        egui::DragValue::new(&mut controls.noise.offset[1]).prefix("Z: ")
                    ).drag_released();
                });
                ui.end_row();

                ui.label("Seed");
                brush_reload |= ui.add(
                    egui::DragValue::new(&mut controls.noise.seed)
                ).drag_released();
                ui.end_row();
            },
            _ => {}
        };
        controls.brush_reload = brush_reload;


        ui.label("Preview");
        ui.image(egui::TextureId::User(controls.brush_texture.id), [100.0, 100.0]);
        ui.end_row();
    });
}


