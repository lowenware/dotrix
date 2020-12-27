use dotrix::{
    ecs::{ Mut },
    egui::{ Egui, SidePanel, Slider },
    services::{ Renderer },
};


pub struct Editor {
    name: String,
    value: u32,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            name: String::from("Marching Cubes"),
            value: 64,
        }
    }
}

pub fn ui(mut editor: Mut<Editor>, renderer: Mut<Renderer>) {
    let egui = renderer.overlay
        .as_ref()
        .expect("Renderer does not contain an Overlay instance")
        .provider::<Egui>();

    SidePanel::left("side_panel", 200.0).show(&egui.ctx, |ui| {
        ui.heading("Terrain");
        ui.horizontal(|ui| {
            ui.label("Project: ");
            ui.text_edit_singleline(&mut editor.name);
        });
        ui.add(Slider::u32(&mut editor.value, 0..=120).text("Chunk size"));
        if ui.button("Update").clicked {
            editor.value += 1;
        }
        // ui.label(format!("Hello '{}', age {}", name, age));
    });
}

