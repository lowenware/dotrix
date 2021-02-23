use super::{ Color, Light };

#[derive(Clone, Debug)]
/// Component to be added to entities
pub struct AmbientLight {
    pub color: Color,
    pub intensity: f32,
}

impl AmbientLight {
    pub fn new(color: Color, intensity: f32) -> Self {
        Self {
            color,
            intensity,
        }
    }
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: Color::white(),
            intensity: 0.2,
        }
    }
}

impl Light<Color> for AmbientLight {
    fn to_raw(&self) -> Color {
        self.color.mul_f32(self.intensity)
    }
}