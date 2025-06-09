mod renderer;

use crate::math::{Quat, Vec3};
use crate::models::Transform3D;
use crate::{Color, Mesh};

pub use renderer::RenderSkyDome;

pub struct SkyDome {
    /// horizon color
    pub horizon_color: Color<f32>,
    /// zenit color
    pub zenith_color: Color<f32>,
    /// Mesh
    pub mesh: Mesh,
    /// Mesh transformation
    pub transform: Transform3D,
    /// Size
    pub size: f32,
}

pub struct SkyLevel {
    /// Sky color
    pub color: Color<f32>,
    /// Level where the color starts
    pub level: f32,
}

impl SkyDome {
    pub fn new(size: f32, horizon_color: Color<f32>, zenith_color: Color<f32>) -> Self {
        let mesh = Mesh::hemisphere("Hemisphere", 64, 32);
        let transform = Transform3D {
            scale: Vec3::new(size, size, size),
            translate: Vec3::new(0.0, -100.0, 0.0),
            rotate: Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), -(std::f32::consts::PI / 2.0)),
        };
        Self {
            mesh,
            transform,
            size,
            horizon_color,
            zenith_color,
        }
    }
}

impl Default for SkyDome {
    fn default() -> Self {
        let horizon_color = Color::rgba(0.67, 0.855, 0.905, 1.0);
        let zenith_color = Color::rgba(0.0, 0.44, 0.75, 1.0);
        Self::new(800.0, horizon_color, zenith_color)
    }
}
