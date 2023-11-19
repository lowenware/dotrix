use dotrix_gpu as gpu;
// use dotrix_input::Input;
use dotrix_log as log;
use dotrix_mesh::Mesh;
use dotrix_types::{vertex, Color, Frame, Id, TexUV};
use fontdue::layout::VerticalAlign;

use crate::{overlay, Direction, Rect, State, Style, Viewport, Widget};

pub struct View<'s> {
    /// Style reference for the current view
    pub style: &'s Style,
    /// Outer rectangle
    pub outer: Rect,
    /// Inner rectangle
    pub inner: Rect,
}

impl<'s> View<'s> {
    pub fn new(outer: Rect, style: &'s Style) -> Self {
        let inner = outer.inner(&style.padding);
        Self {
            style,
            outer,
            inner,
        }
    }

    pub fn compose(
        &mut self,
        state: Option<State>,
        frame_width: f32,
        frame_height: f32,
        scale_factor: f32,
    ) -> Option<Widget> {
        self.style.background.as_ref().map(|background| {
            let (x0, x1) =
                Self::calculate_side(frame_width, self.outer.horizontal, self.outer.width);
            let (y0, y1) =
                Self::calculate_side(frame_height, self.outer.vertical, self.outer.height);

            let positions = vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
            let uvs = vec![
                background.uvs.top_left.pack(),
                background.uvs.top_right.pack(),
                background.uvs.bottom_right.pack(),
                background.uvs.bottom_left.pack(),
            ];
            let colors = vec![
                background.color.top_left.into(),
                background.color.top_right.into(),
                background.color.bottom_right.into(),
                background.color.bottom_left.into(),
            ];
            let indices = vec![0, 3, 1, 1, 3, 2];
            let mut mesh = Mesh::new("view"); // TODO: use ID here
            mesh.set_vertices::<overlay::Position>(positions);
            mesh.set_vertices::<vertex::TexUV>(uvs);
            mesh.set_vertices::<Color<u8>>(colors);
            mesh.set_indices(indices);

            Widget {
                rect: self.outer.clone(),
                mesh,
                texture: background.texture,
            }
        })
    }

    pub fn update_size(&mut self, content_width: f32, content_height: f32) {
        self.inner.width = content_width;
        self.inner.height = content_height;
        match self.style.direction {
            Direction::Horizontal => {
                let outer_width =
                    content_width + self.style.padding.left + self.style.padding.right;
                if outer_width < self.outer.width {
                    self.outer.width = outer_width;
                }
            }
            Direction::Vertical => {
                let outer_height =
                    content_height + self.style.padding.top + self.style.padding.bottom;
                if outer_height < self.outer.height {
                    self.outer.height = outer_height;
                }
            }
        };
    }

    pub fn viewport(&self) -> Viewport {
        let direction = self.style.direction;
        Viewport {
            rect: self.inner.clone(),
            content_width: 0.0,
            content_height: 0.0,
            direction,
        }
    }

    fn calculate_side(frame_side: f32, offset: f32, side: f32) -> (f32, f32) {
        if offset < 0.0 {
            let v = frame_side + offset;
            (v - side, v)
        } else {
            (offset, offset + side)
        }
    }
}

/*
impl Compose for View {
    fn compose<'c, 'i, 'f>(
        &self,
        rect: &Rect,
        composer: &mut crate::composer::Composer<'c, 'i, 'f>,
    ) {
        let outer_rect = rect.inner(&self.style.margin);
        let inner_rect = outer_rect.inner(&self.style.padding);

        let pushed = if let Some(background) = self.style.background.as_ref() {
            let frame_width = composer.frame.width as f32;
            let frame_height = composer.frame.height as f32;

            let (x0, x1) =
                Self::calculate_side(frame_width, outer_rect.horizontal, outer_rect.width);
            let (y0, y1) =
                Self::calculate_side(frame_height, outer_rect.vertical, outer_rect.height);

            let (builder, pushed) = composer.builder(outer_rect, background.texture);

            log::debug!("Builder pushed: {}", pushed);
            let vertex_base = builder.positions.len() as u32;

            builder
                .positions
                .extend_from_slice(&[[x0, y0], [x1, y0], [x1, y1], [x0, y1]]);
            builder.uvs.extend_from_slice(&[
                background.uvs.top_left.pack(),
                background.uvs.top_right.pack(),
                background.uvs.bottom_right.pack(),
                background.uvs.bottom_left.pack(),
            ]);
            builder.colors.extend_from_slice(&[
                background.color.top_left.into(),
                background.color.top_right.into(),
                background.color.bottom_right.into(),
                background.color.bottom_left.into(),
            ]);
            builder.indices.extend_from_slice(&[
                0 + vertex_base,
                3 + vertex_base,
                1 + vertex_base,
                1 + vertex_base,
                3 + vertex_base,
                2 + vertex_base,
            ]);
            pushed
        } else {
            false
        };

        for child in self.children.iter() {
            child.compose(&inner_rect, composer);
        }

        if pushed {
            if let Some(builder) = composer.builder_pop() {
                composer.add_widget(builder.into())
            }
        }
    }
}

 */

/*
    pub fn tesselate(mut self, child: T) -> Self {
        self.children.push(Box::new(child));
        self
    }
*/

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
