use dotrix_egui::native as egui;

/// 
pub struct Editor {

}

impl Default for Editor {
    fn default() -> Self {
        Self {

        }
    }
}


impl Editor {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Terrain Editor");
    }
}

