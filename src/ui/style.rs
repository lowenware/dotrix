use dotrix_gpu as gpu;
use dotrix_types::vertex::TexUV;
use dotrix_types::{Color, Id};

use crate::Font;

#[derive(Default)]
pub struct Style {
    pub background: Option<Background>,
    pub direction: Direction,
    pub margin: Spacing,
    pub padding: Spacing,
    pub width: Size,
    pub height: Size,
}

pub struct FontStyle<'f> {
    pub texture: Id<gpu::TextureView>,
    pub font: &'f Font,
    pub color: Color<u8>,
}

pub struct Background {
    pub color: Corners<Color<u8>>,
    pub texture: Id<gpu::TextureView>,
    pub uvs: Corners<TexUV>,
}

pub struct Corners<T: Clone> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_right: T,
    pub bottom_left: T,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    Vertical,
    Horizontal,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Vertical
    }
}

#[derive(Default, Debug, Clone)]
pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl From<f32> for Spacing {
    fn from(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Size {
    /// size required by a widget
    Auto,
    /// exact size
    Px(f32),
    /// use available space, according to a factor 0.0..1.0 of the parent size
    Grow(f32),
}

impl Default for Size {
    fn default() -> Self {
        Size::Auto
    }
}

#[derive(Default, Debug, Clone)]
pub struct Text {
    pub font_size: f32,
    pub color: Color<u8>,
}

impl Default for Background {
    fn default() -> Self {
        let white = Color::white();
        let uv = TexUV::default();
        Self {
            color: Corners {
                top_left: white,
                top_right: white,
                bottom_right: white,
                bottom_left: white,
            },
            texture: Id::default(),
            uvs: Corners {
                top_left: uv,
                top_right: uv,
                bottom_right: uv,
                bottom_left: uv,
            },
        }
    }
}

impl Background {
    pub fn from_color(color: Color<u8>) -> Self {
        Self {
            color: Corners {
                top_left: color,
                top_right: color,
                bottom_right: color,
                bottom_left: color,
            },
            ..Default::default()
        }
    }
}

impl<T: Clone> From<&Corners<T>> for [T; 4] {
    fn from(entry: &Corners<T>) -> Self {
        [
            entry.top_left.clone(),
            entry.top_right.clone(),
            entry.bottom_right.clone(),
            entry.bottom_left.clone(),
        ]
    }
}
