//! data structure and constructors
use std::convert::From;
use std::ops::{Index, IndexMut};

/// RGBA Color.
#[derive(Default, Copy, Clone, Debug)]
pub struct Color {
    /// Red channel. Should be in range from 0 to 1.
    pub r: f32,
    /// Green channel. Should be in range from 0 to 1.
    pub g: f32,
    /// Blue channel. Should be in range from 0 to 1.
    pub b: f32,
    /// Alpha channel. Should be in range from 0 to 1.
    pub a: f32,
}

impl Color {
    /// RGB Constructor, values should be in range from 0 to 1. Alpha will be 1.
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// RGBA Constructor. Values should be in range from 0 to 1.
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Red color (r: 1.0, g: 0.0, b: 0.0)
    pub fn red() -> Self {
        Self::rgb(1.0, 0.0, 0.0)
    }

    /// Grey color
    pub fn grey() -> Self {
        Self::rgb(0.6, 0.6, 0.6)
    }

    /// Green color (r: 0.0, g: 1.0, b: 0.0)
    pub fn green() -> Self {
        Self::rgb(0.0, 1.0, 0.0)
    }

    /// Blue color (r: 0.0, g: 0.0, b: 1.0)
    pub fn blue() -> Self {
        Self::rgb(0.0, 0.0, 1.0)
    }

    /// Cyan color (r: 0.0, g: 1.0, b: 1.0)
    pub fn cyan() -> Self {
        Self::rgb(0.0, 1.0, 1.0)
    }

    /// Magenta color (r: 1.0, g: 0.0, b: 1.0)
    pub fn magenta() -> Self {
        Self::rgb(1.0, 0.0, 1.0)
    }

    /// Yellow color (r: 1.0, g: 1.0, b: 0.0)
    pub fn yellow() -> Self {
        Self::rgb(1.0, 1.0, 0.0)
    }

    /// White color (r: 1.0, g: 1.0, b: 1.0)
    pub fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }

    /// Black color (r: 0.0, g: 0.0, b: 0.0)
    pub fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }

    /// Orange color (r: 1.0, g: 0.5, b: 0.0)
    pub fn orange() -> Self {
        Self::rgb(1.0, 0.5, 0.0)
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;

    /// Multiply Color by f32. Result is not clamped.
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl Index<i32> for Color {
    type Output = f32;

    fn index(&self, index: i32) -> &Self::Output {
        match index {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            3 => &self.a,
            _ => panic!("Color index is out of range."),
        }
    }
}

impl IndexMut<i32> for Color {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        match index {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            3 => &mut self.a,
            _ => panic!("Color index is out of range."),
        }
    }
}

impl From<[f32; 3]> for Color {
    fn from(rgb: [f32; 3]) -> Self {
        Self {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 1.0,
        }
    }
}

impl From<[f32; 4]> for Color {
    fn from(rgba: [f32; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(rgba: Color) -> Self {
        [rgba.r, rgba.g, rgba.b, rgba.a]
    }
}

impl From<Color> for [f32; 3] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b]
    }
}
