use dotrix::egui;

pub fn tool_grid(id: &str) -> egui::Grid {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
}
