use dotrix::egui::native as egui;

pub struct Controls {
}

impl Default for Controls {
    fn default() -> Self {
        Self {
        }
    }
}

pub fn show(ui: &mut egui::Ui, _controls: &mut Controls) {
    ui.label("Objects UI");
}


