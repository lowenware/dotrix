/// Light component of different types and settings
use super::storage::GenericLight;
pub use dotrix_pbr::Light;

/// Directional light uniform data
pub(super) struct DirectionalLight {
    /// Light direction
    pub(super) direction: [f32; 4],
    /// Light color
    pub(super) color: [f32; 4],
}

impl From<DirectionalLight> for GenericLight {
    fn from(src: DirectionalLight) -> Self {
        Self {
            position: Default::default(),
            direction: src.direction,
            color: src.color,
            parameters: [0., 0., 0., 0.],
            kind: 1,
            padding: Default::default(),
        }
    }
}

/// Point light uniform data
pub(super) struct PointLight {
    /// Light source position
    pub(super) position: [f32; 4],
    /// Light color
    pub(super) color: [f32; 4],
    /// Constant light attenuation
    pub(super) a_constant: f32,
    /// Linear light attenuation
    pub(super) a_linear: f32,
    /// Quadratic light attenuation
    pub(super) a_quadratic: f32,
}

impl From<PointLight> for GenericLight {
    fn from(src: PointLight) -> Self {
        Self {
            position: src.position,
            direction: Default::default(),
            color: src.color,
            parameters: [src.a_constant, src.a_linear, src.a_quadratic, 0.],
            kind: 2,
            padding: Default::default(),
        }
    }
}

/// Simple light uniform data
pub(super) struct SimpleLight {
    /// Light source position
    pub(super) position: [f32; 4],
    /// Light color
    pub(super) color: [f32; 4],
}

impl From<SimpleLight> for GenericLight {
    fn from(src: SimpleLight) -> Self {
        Self {
            position: src.position,
            direction: Default::default(),
            color: src.color,
            parameters: Default::default(),
            kind: 3,
            padding: Default::default(),
        }
    }
}

/// Spot Light uniform data
pub(super) struct SpotLight {
    /// Light source position
    pub(super) position: [f32; 4],
    /// Light source direction
    pub(super) direction: [f32; 4],
    /// Light source color
    pub(super) color: [f32; 4],
    /// Light source cut off
    pub(super) cut_off: f32,
    /// Light source outer cut off
    pub(super) outer_cut_off: f32,
}

impl From<SpotLight> for GenericLight {
    fn from(src: SpotLight) -> Self {
        Self {
            position: src.position,
            direction: src.direction,
            color: src.color,
            parameters: [src.cut_off, src.outer_cut_off, 0., 0.],
            kind: 4,
            padding: Default::default(),
        }
    }
}
