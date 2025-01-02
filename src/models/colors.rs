//! data structure and constructors
use super::VertexAttribute;
use crate::graphics::Format;

pub trait Channel {
    fn value(value: f32) -> Self;
}

impl Channel for f32 {
    fn value(value: f32) -> Self {
        value.clamp(0.0, 1.0)
    }
}

impl Channel for u8 {
    fn value(value: f32) -> Self {
        (value.clamp(0.0, 1.0) * 255.0) as u8
    }
}

/// RGBA Color.
#[derive(Default, Copy, Clone, Debug)]
pub struct Color<T>
where
    T: Channel + Copy,
{
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T> Color<T>
where
    T: Channel + Copy,
{
    /// RGB Constructor, values should be in range from 0 to 1. Alpha will be 1.
    pub fn rgb(r: T, g: T, b: T) -> Self {
        Self::rgba(r, g, b, T::value(1.0))
    }

    /// RGBA Constructor. Values should be in range from 0 to 1.
    pub fn rgba(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }

    /// Red color (r: 1.0, g: 0.0, b: 0.0)
    pub fn red() -> Self {
        Self::rgb(T::value(1.0), T::value(0.0), T::value(0.0))
    }

    /// Grey color
    pub fn grey() -> Self {
        Self::rgb(T::value(0.6), T::value(0.6), T::value(0.6))
    }

    /// Green color (r: 0.0, g: 1.0, b: 0.0)
    pub fn green() -> Self {
        Self::rgb(T::value(0.0), T::value(1.0), T::value(0.0))
    }

    /// Blue color (r: 0.0, g: 0.0, b: 1.0)
    pub fn blue() -> Self {
        Self::rgb(T::value(0.0), T::value(0.0), T::value(1.0))
    }

    /// Cyan color (r: 0.0, g: 1.0, b: 1.0)
    pub fn cyan() -> Self {
        Self::rgb(T::value(0.0), T::value(1.0), T::value(1.0))
    }

    /// Magenta color (r: 1.0, g: 0.0, b: 1.0)
    pub fn magenta() -> Self {
        Self::rgb(T::value(1.0), T::value(0.0), T::value(1.0))
    }

    /// Yellow color (r: 1.0, g: 1.0, b: 0.0)
    pub fn yellow() -> Self {
        Self::rgb(T::value(1.0), T::value(1.0), T::value(0.0))
    }

    /// White color (r: 1.0, g: 1.0, b: 1.0)
    pub fn white() -> Self {
        Self::rgb(T::value(1.0), T::value(1.0), T::value(1.0))
    }

    /// Black color (r: 0.0, g: 0.0, b: 0.0)
    pub fn black() -> Self {
        Self::rgb(T::value(0.0), T::value(0.0), T::value(0.0))
    }

    /// Orange color (r: 1.0, g: 0.5, b: 0.0)
    pub fn orange() -> Self {
        Self::rgb(T::value(1.0), T::value(0.5), T::value(0.0))
    }
}

impl VertexAttribute for Color<f32> {
    type Raw = [f32; 4];
    fn name() -> &'static str {
        "Color"
    }
    fn format() -> Format {
        Format::Float32x4
    }
    fn pack(&self) -> Self::Raw {
        self.into()
    }
}

impl std::ops::Mul<f32> for Color<f32> {
    type Output = Self;

    /// Multiply Color by f32. Result is not clamped.
    fn mul(self, rhs: f32) -> Self::Output {
        Self::from([self.r * rhs, self.g * rhs, self.b * rhs, self.a * rhs])
    }
}

impl From<[f32; 3]> for Color<f32> {
    fn from(rgb: [f32; 3]) -> Self {
        Self::rgb(rgb[0], rgb[1], rgb[2])
    }
}

impl From<[f32; 4]> for Color<f32> {
    fn from(rgba: [f32; 4]) -> Self {
        Self::rgba(rgba[0], rgba[1], rgba[2], rgba[3])
    }
}

impl From<&Color<f32>> for [f32; 4] {
    fn from(color: &Color<f32>) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl From<&Color<f32>> for [f32; 3] {
    fn from(color: &Color<f32>) -> Self {
        [color.r, color.g, color.b]
    }
}

impl From<&Color<u8>> for Color<f32> {
    fn from(color: &Color<u8>) -> Self {
        Color {
            r: (color.r as f32) / 255.0,
            g: (color.g as f32) / 255.0,
            b: (color.b as f32) / 255.0,
            a: (color.a as f32) / 255.0,
        }
    }
}

impl VertexAttribute for Color<u8> {
    type Raw = u32;
    fn name() -> &'static str {
        "Color"
    }
    fn format() -> Format {
        Format::Uint32
    }
    fn pack(&self) -> Self::Raw {
        self.into()
    }
}

impl From<&Color<u8>> for u32 {
    fn from(color: &Color<u8>) -> Self {
        (color.r as u32) << 24 | (color.g as u32) << 16 | (color.b as u32) << 8 | color.a as u32
    }
}

impl From<u32> for Color<u8> {
    fn from(color: u32) -> Self {
        Self {
            r: (color >> 24 & 0x0FF) as u8,
            g: (color >> 16 & 0x0FF) as u8,
            b: (color >> 8 & 0x0FF) as u8,
            a: (color & 0x0FF) as u8,
        }
    }
}

impl From<[u8; 3]> for Color<u8> {
    fn from(color: [u8; 3]) -> Self {
        Self {
            r: color[0],
            g: color[1],
            b: color[2],
            a: 0xFF,
        }
    }
}
