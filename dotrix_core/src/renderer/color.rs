// TODO: move Color somewhere else. Consider 32bit (0-255) too
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
/// RGBA Color.
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
    /// RGB Constructor, alpha will be 1.
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// RGB Constructor.
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// White color (r: 1.0, g: 1.0, b: 1.0, a: 1.0)
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    /// Black color (r: 0.0, g: 0.0, b: 0.0, a: 0.0)
    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
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
