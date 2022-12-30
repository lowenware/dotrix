use dotrix_gpu as gpu;
use dotrix_types::vertex::TexUV;
use dotrix_types::{Color, Id};

#[derive(Default)]
pub struct Style {
    pub background: Option<Background>,
    pub margin: Spacing,
    pub padding: Spacing,
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

#[derive(Default, Debug, Clone)]
pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
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
