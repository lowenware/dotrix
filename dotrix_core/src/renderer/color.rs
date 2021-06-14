//! data structure and constructors
use std::convert::From;
use std::ops::{ Index, IndexMut };

/// RGBA Color.
#[derive(Copy, Clone, Debug)]
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

    /// Multiply Color by f32. Result is not clamped.
    pub fn mul_f32(self, rhs: f32) -> Self {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }

    /// Create color from array
    pub fn from_f32_3(rgb: [f32; 3]) -> Self {
        Self::rgb(rgb[0], rgb[1], rgb[2])
    }

    /// Create color from array
    pub fn from_f32_4(rgba: [f32; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    /// Convert color to array. Alpha is ignored.
    pub fn to_f32_3(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    /// Convert color to array.
    pub fn to_f32_4(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
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
        Self::from_f32_3(rgb)
    }
}

impl From<[f32; 4]> for Color {
    fn from(rgba: [f32; 4]) -> Self {
        Self::from_f32_4(rgba)
    }
}

/*
impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        self.to_f32_3()
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        self.to_f32_4()
    }
}
*/
