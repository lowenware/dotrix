use dotrix::egui::native as egui;
use dotrix::Id;
use dotrix::assets::Texture;
use dotrix::egui::extras::FileDialog;
use dotrix::math::Vec2u;
use dotrix::terrain::{ Component, Lod, Noise };
use crate::brush::{
    Brush,
    INTENSITY as BRUSH_INTENSITY,
    SIZE as BRUSH_SIZE,
    Mode as BrushMode,
    Shape as BrushShape,
};

pub enum HeightMapAction {
    Reset,
    Import,
    Export,
}

pub struct Controls {
    /// Terrain Component Type
    pub component: Component,
    /// Level of Details
    pub lod: u32,
    /// Number of components per X
    pub tiles_per_x: u32,
    /// Number of components per Z (not implemented yet)
    pub tiles_per_z: u32,
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
    /// Reload brush
    pub brush_reload: bool,
    /// Noise configuration
    pub noise: Noise,
    /// Id of the brush texture
    pub brush_texture: Id<Texture>,
    /// Reload terrain map
    pub map_reload: bool,

    pub height_map_action: Option<HeightMapAction>,

    pub sizes: Vec<(u32, String)>,

    pub save_file_dialog: FileDialog,
}

impl Controls {
    pub fn set_brush_texture(&mut self, brush_texture: Id<Texture>) {
        self.brush_texture = brush_texture;
    }
}

impl Default for Controls {
    fn default() -> Self {
        let mut result = Self {
            component: Component::Standard,
            lod: 0,
            tiles_per_x: 2,
            tiles_per_z: 2,
            y_scale: 512.0,
            brush_mode: BrushMode::Elevate,
            brush_shape: BrushShape::Radial,
            brush_intensity: BRUSH_INTENSITY,
            brush_size: BRUSH_SIZE,
            brush_reload: true,
            brush_texture: Id::default(),
            noise: Noise::default(),
            map_reload: true,
            height_map_action: None,
            sizes: Vec::new(),
            save_file_dialog: FileDialog::save_file(None),
        };
        result.calculate_sizes();
        result
    }
}

impl Controls {
    fn calculate_sizes(&mut self) {
        self.sizes = (1..=6).map(|i| {
            let tiles_per_x = 2 * i;
            let units = self.component.units_per_side() as u32;
            let lod = Lod::from_level(self.lod);
            let size = tiles_per_x * units * lod.scale();
            (tiles_per_x, format!("{}", size))
        }).collect::<Vec<_>>();
    }
}


pub fn show(ctx: &egui::CtxRef, ui: &mut egui::Ui, controls: &mut Controls, brush: &mut Brush) {

    egui::CollapsingHeader::new("Properties")
        .default_open(true)
        .show(ui, |ui| show_properties(ctx, ui, controls));
    ui.separator();

    egui::CollapsingHeader::new("Brush")
        .default_open(true)
        .show(ui, |ui| show_brush(ui, controls, brush));
    ui.separator();
}


fn show_properties(ctx: &egui::CtxRef, ui: &mut egui::Ui, controls: &mut Controls) {
    super::tool_grid("heightmap_properties").show(ui, |ui| {
        ui.label("Height Map");
        ui.horizontal(|ui| {
            if ui.button("New").clicked() {
                controls.height_map_action = Some(HeightMapAction::Reset);
            }
            if ui.button("Import").clicked() {
                controls.height_map_action = Some(HeightMapAction::Import);
            }

            if ui.button("Export").clicked() {
                controls.save_file_dialog.open()
            }

            if controls.save_file_dialog.show(ctx).selected() {
                controls.height_map_action = Some(HeightMapAction::Export);
            }

        });
        ui.end_row();

        let quads_per_side = controls.component.units_per_side() as u32;
        let mut size_x = controls.tiles_per_x * quads_per_side + 1;
        let limit_x = size_x;
        let mut size_z = controls.tiles_per_z * quads_per_side + 1;

        let component = controls.component;
        ui.label("Component");
        egui::ComboBox::from_id_source("terrain_component")
            .selected_text(controls.component.as_str())
            .show_ui(ui, |ui| {
                for component in Component::slice() {
                    ui.selectable_value(&mut controls.component, component, component.as_str());
                }
            });
        ui.end_row();

        let lod = controls.lod;
        ui.label("Level of Details");
            let lods = [
                "1 / 1 (High)",
                "1 / 2",
                "1 / 4",
                "1 / 8 (Low)",
            ];
            egui::ComboBox::from_id_source("terrain_lod")
                .selected_text(lods[controls.lod as usize])
                .show_ui(ui, |ui| {
                    for (lod, label) in lods.iter().enumerate() {
                        ui.selectable_value(&mut controls.lod, lod as u32, label);
                    }
                });
        ui.end_row();

        if lod != controls.lod || component != controls.component {
            controls.calculate_sizes();
            controls.map_reload = true;
        }

        let tiles_per_x = controls.tiles_per_x;

        ui.label("Size");
        ui.horizontal(|ui| {
            for (tiles_per_x, label) in controls.sizes.iter() {
                ui.selectable_value(&mut controls.tiles_per_x, *tiles_per_x, label);
            }
        });

        if tiles_per_x != controls.tiles_per_x {
            controls.map_reload = true;
        }
        ui.end_row();

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
    let mut brush_reload = false;
    super::tool_grid("brush").show(ui, |ui| {

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

        let brush_shape = controls.brush_shape;

        ui.label("Mode");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut controls.brush_shape, BrushShape::Radial, "Radial");
            ui.selectable_value(&mut controls.brush_shape, BrushShape::Noise, "Noise");
        });
        ui.end_row();

        if brush_shape != controls.brush_shape {
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

        ui.label("Preview");
        ui.image(egui::TextureId::User(controls.brush_texture.id), [100.0, 100.0]);
        ui.end_row();
    });

    controls.brush_reload = brush_reload;
}


