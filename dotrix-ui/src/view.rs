use dotrix_gpu as gpu;
// use dotrix_input::Input;
use dotrix_log as log;
use dotrix_types::{vertex, Color, Frame, Id, TexUV};

use crate::composer::Compose;
use crate::{Rect, Style};

#[derive(Default)]
pub struct View {
    pub style: Style,
    pub children: Vec<Box<dyn Compose>>,
}

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

            let (builder, pushed) = composer.builder(background.texture);

            log::debug!("Builder pushed: {}", pushed);

            let (x0, x1) =
                Self::calculate_side(frame_width, outer_rect.horizontal, outer_rect.width);
            let (y0, y1) =
                Self::calculate_side(frame_height, outer_rect.vertical, outer_rect.height);

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

impl View {
    fn calculate_side(frame_side: f32, offset: f32, side: f32) -> (f32, f32) {
        if offset < 0.0 {
            let v = frame_side + offset;
            (v - side, v)
        } else {
            (offset, offset + side)
        }
    }

    pub fn new(style: Style) -> Self {
        Self {
            style,
            ..Default::default()
        }
    }

    pub fn append<T: Compose + 'static>(&mut self, child: T) {
        self.children.push(Box::new(child));
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
