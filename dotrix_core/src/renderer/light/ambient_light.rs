use super::{ Color, Light };

/// Component to be added to entities
#[derive(Clone, Debug)]
pub struct AmbientLight {
    /// Light source color
    pub color: Color,
    /// Light source intensity
    pub intensity: f32,
}

impl AmbientLight {
    /// Constructs new light source component
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
