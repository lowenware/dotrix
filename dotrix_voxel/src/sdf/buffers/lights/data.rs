/// Light component of different types and settings
use super::storage::GenericLight;
use dotrix_core::Color;
use dotrix_math::Vec3;

#[allow(dead_code)]
pub enum Light {
    Ambient {
        /// Light source color
        color: Color,
        /// Light source intensity
        intensity: f32,
    },
    Directional {
        /// Light source color
        color: Color,
        /// Light source direction
        direction: Vec3,
        /// Light source intensity
        intensity: f32,
        /// Is light source enabled
        enabled: bool,
    },
    Simple {
        /// Light color
        color: Color,
        /// Light source position
        position: Vec3,
        /// Light intensity
        intensity: f32,
        /// Is light source enabled
        enabled: bool,
    },
    Point {
        /// Light color
        color: Color,
        /// Light source position
        position: Vec3,
        /// Light source intencity
        intensity: f32,
        /// Is light source enabled
        enabled: bool,
        /// Constant light
        constant: f32,
        /// Linear light
        linear: f32,
        /// Quadratic light
        quadratic: f32,
    },
    Spot {
        /// Light source color
        color: Color,
        /// Light source position
        position: Vec3,
        /// Light source direction
        direction: Vec3,
        /// Light source intensity
        intensity: f32,
        /// Is light source enabled
        enabled: bool,
        /// Light source cut off
        cut_off: f32,
        /// Light source outer cut off
        outer_cut_off: f32,
    },
}

#[allow(dead_code)]
impl Light {
    pub fn ambient() -> Self {
        Light::Ambient {
            color: Color::white(),
            intensity: 0.8,
        }
    }

    pub fn directional() -> Self {
        Light::Directional {
            enabled: true,
            direction: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
        }
    }

    pub fn simple() -> Self {
        Light::Simple {
            enabled: true,
            position: Vec3::new(0.0, 1000.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
        }
    }

    pub fn point() -> Self {
        Light::Point {
            enabled: true,
            position: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
            constant: 1.0,
            linear: 0.35,
            quadratic: 0.44,
        }
    }

    pub fn spot() -> Self {
        Light::Spot {
            enabled: true,
            position: Vec3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
            cut_off: 0.8,
            outer_cut_off: 0.65,
        }
    }
}

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
