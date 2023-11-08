use dotrix_mesh::Mesh;
use dotrix_types::{vertex, Id, Color};
use dotrix_gpu as gpu;
use crate::{Rect, State, Widget, overlay, font};
use crate::style::FontStyle;

pub struct Label {
    rect: Rect,
    _mesh: Option<Mesh>,
    _texture: Id<gpu::TextureView>,
}

impl Label {
    pub fn new<'s>(id: Option<&str>, mut rect: Rect, style: &FontStyle<'s>, text: &str) -> Self {
        let mesh = Self::build_mesh(id, &mut rect, style, text);
        let texture = style.texture;
        Self {
            rect,
            _mesh: mesh,
            _texture: texture,
        }
    }

    fn build_mesh<'s>(id: Option<&str>, rect: &mut Rect, style: &FontStyle<'s>, text: &str) -> Option<Mesh> {
        let text_len = text.len();
        if text_len == 0 {
            return None;
        }

        let vertices_count = text_len * 4;
        let mut positions = Vec::with_capacity(vertices_count);
        let colors: Vec<u32> = vec![style.color.into(); vertices_count];
        let mut uvs = Vec::with_capacity(vertices_count);
        let mut indices = Vec::with_capacity(text_len * 6);
        let mut mesh = Mesh::new(id.unwrap_or("label"));

        let mut cursor = font::Cursor::default();

        for word in text.split_inclusive(&[' ', '\n']) {
            style.font.place_word(rect, &mut cursor, word);            

            for character in word.chars() {
                // positions and uvs
                let (ch_pos, ch_uvs) = style.font.tesselate(rect, &mut cursor, character);
                positions.extend(&ch_pos);
                uvs.extend(&ch_uvs);

                // indices
                let indices_offset = indices.len() as u32;
                indices.extend_from_slice(&[
                    indices_offset + 1,
                    indices_offset,
                    indices_offset + 3,
                    indices_offset + 1,
                    indices_offset + 3,
                    indices_offset + 2,
                ]);

            }
        }

        mesh.set_vertices::<overlay::Position>(positions);
        mesh.set_vertices::<vertex::TexUV>(uvs);
        mesh.set_vertices::<Color<u8>>(colors);
        mesh.set_indices(indices);


        Some(mesh)
    }

    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    pub fn compose(
        &mut self,
        _state: Option<State>,
        _frame_width: f32,
        _frame_height: f32,
        _scale_factor: f32,
    ) -> Option<Widget> {
        todo!("Label::compose is not implemented!");
    }
}