use crate::Asset;

pub struct ColorMap {
    name: String,
    colors: Vec<[u8; 3]>,
    moisture_blend_factor: f32,
}

impl ColorMap {
    pub fn new(name: impl Into<String>, colors: Vec<[u8; 3]>, moisture_blend_factor: f32) -> Self {
        Self {
            name: name.into(),
            colors,
            moisture_blend_factor,
        }
    }

    pub fn color(&self, height: f32, moisture: f32) -> [u8; 3] {
        let color_count = self.colors.len() as f32;
        let color_step = 1.0 / color_count;
        let moisture_blend =
            moisture * self.moisture_blend_factor - (self.moisture_blend_factor / 2.0);
        let color_height = (height + moisture_blend).clamp(0.0, 1.0);
        let color_index = (color_height / color_step).floor() as usize;

        self.colors[if color_index < self.colors.len() {
            color_index
        } else {
            self.colors.len() - 1
        }]
    }
}

impl Asset for ColorMap {
    fn name(&self) -> &str {
        &self.name
    }
}
