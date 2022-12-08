use dotrix_gpu as gpu;
// use dotrix_input::Input;
use dotrix_mesh::Mesh;
use dotrix_types::{vertex, Color, Frame, Id};

use crate::{Position, Rect, Widget};

#[derive(Default)]
pub struct View {
    pub style: Style,
    pub children: Vec<View>,
}

impl View {
    fn calculate_side(frame_side: f32, offset: f32, side: f32) -> (f32, f32) {
        if offset < 0.0 {
            let v = frame_side + offset;
            (v - side, v)
        } else {
            (offset, offset + side)
        }
    }

    pub fn compose<'a>(&'a mut self, rect: Rect, /*_input: &Input,*/ frame: &Frame) -> Widget {
        let (x0, x1) = Self::calculate_side(frame.width as f32, rect.horizontal, rect.width);
        let (y0, y1) = Self::calculate_side(frame.height as f32, rect.vertical, rect.height);

        let positions = vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
        let tex_uv = vec![[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]];
        let colors: Vec<u32> = vec![
            Color::<u8>::rgba(80, 80, 80, 200).into(),
            Color::<u8>::rgba(80, 80, 80, 200).into(),
            Color::<u8>::rgba(80, 80, 80, 200).into(),
            Color::<u8>::rgba(80, 80, 80, 200).into(),
        ];
        let indices = vec![0, 3, 1, 1, 3, 2];
        let mut mesh = Mesh::new(String::from("View"));
        mesh.set_vertices::<Position>(positions);
        mesh.set_vertices::<vertex::TexUV>(tex_uv);
        mesh.set_vertices::<Color<u8>>(colors);
        mesh.set_indices(indices);

        Widget {
            mesh,
            texture: Id::default(),
        }
    }
}

/*
pub struct Iter<'a> {
    view: &'a mut View,
    stack: Vec<Iter<'a>>
}

impl<'a> Iterator for Iter<'a> {
    type Item = Widget;
    fn next(&mut self) -> Option<Self::Item> {

    }
}*/

pub enum Background {
    Color(Color<u8>),
    Image(Id<gpu::TextureView>),
    // Gradient(...)
}

#[derive(Default)]
pub struct Style {
    pub border_radius: Option<f32>,
    pub border_top_left_radius: Option<f32>,
    pub border_top_right_radius: Option<f32>,
    pub border_bottom_right_radius: Option<f32>,
    pub border_bottom_left_radius: Option<f32>,
    pub background: Option<Background>,
}

impl Style {
    #[inline(always)]
    fn border_radius(specific: &Option<f32>, global: &Option<f32>) -> f32 {
        specific.unwrap_or_else(|| global.unwrap_or(0.0))
    }

    #[inline(always)]
    pub fn border_top_left_radius(&self) -> f32 {
        Self::border_radius(&self.border_top_left_radius, &self.border_radius)
    }

    #[inline(always)]
    pub fn border_top_right_radius(&self) -> f32 {
        Self::border_radius(&self.border_top_right_radius, &self.border_radius)
    }

    #[inline(always)]
    pub fn border_bottom_right_radius(&self) -> f32 {
        Self::border_radius(&self.border_bottom_right_radius, &self.border_radius)
    }

    #[inline(always)]
    pub fn border_bottom_left_radius(&self) -> f32 {
        Self::border_radius(&self.border_bottom_left_radius, &self.border_radius)
    }
}
