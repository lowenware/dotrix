use std::collections::HashMap;
use std::ops::Range;

use dotrix_types::{vertex, TexUV};
use dotrix_log as log;

use crate::Rect;

pub struct Font {
    size: f32,
    atlas: Atlas,
    map: HashMap<char, usize>,
    glyphs: Vec<Glyph>,
    line_metrics: LineMetrics,
}

impl Font {
    pub fn from_bytes(font_size: f32, charsets: &[Charset], bytes: &[u8]) -> Self {
        let fontdue_font = match fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
        {
            Ok(fontdue_font) => fontdue_font,
            Err(err) => panic!("Could not create font from bytes: {err}"),
        };

        let line_metrics = fontdue_font
            .horizontal_line_metrics(font_size)
            .map(|fontdue_line_metrics| LineMetrics {
                line_ascent: fontdue_line_metrics.ascent,
                line_descent: fontdue_line_metrics.descent,
                line_gap: fontdue_line_metrics.line_gap,
                line_size: fontdue_line_metrics.new_line_size,
            })
            .expect("The font is not populated with metrics");

        let capacity: usize = charsets
            .iter()
            .map(|charset| charset.range())
            .flatten()
            .map(|range| range.len() as usize)
            .sum();

        let mut glyphs = Vec::with_capacity(capacity);
        let mut map = HashMap::with_capacity(capacity);

        for charset in charsets.iter() {
            for range in charset.range().iter() {
                for unicode in range.clone() {
                    if let Some(character) = std::char::from_u32(unicode) {
                        let (metrics, bitmap) = fontdue_font.rasterize(character, font_size);
                        let glyph = Glyph {
                            bitmap,
                            uvs: [vertex::TexUV::default(); 4],
                            rect: Rect {
                                horizontal: metrics.xmin as f32,
                                vertical: metrics.ymin as f32,
                                width: metrics.width as f32,
                                height: metrics.height as f32,
                            },
                            advance_width: metrics.advance_width,
                        };
                        let index = glyphs.len();
                        glyphs.push(glyph);
                        map.insert(character, index);
                    }
                }
            }
        }

        let atlas = Atlas::new(&mut glyphs, font_size);

        Self {
            size: font_size,
            atlas,
            glyphs,
            map,
            line_metrics,
        }
    }

    pub fn line_metrics(&self) -> &LineMetrics {
        &self.line_metrics
    }

    pub fn atlas(&self) -> &Atlas {
        &self.atlas
    }

    /// Returns a tuple of vertices and texture uvs
    pub fn tesselate(
        &self,
        rect: &Rect,
        cursor: &mut Cursor,
        character: char
    ) -> ([[f32; 2]; 4], [[f32; 2]; 4]) {
        log::error!("font::tesselate is not implemented!");
        ([[0.0; 2]; 4], [[0.0; 2]; 4])
    }

    pub fn place_word(&self, rect: &Rect, cursor: &mut Cursor, word: &str) {
        // we are at the beginning of the line, render word even if width is not enough
        if cursor.offset.is_none() {
            return;
        }
        todo!("Place word at the end or go next line, but if not first line");
    }
}

pub struct Atlas {
    bitmap: Vec<u8>,
    width: u32,
    height: u32,
}

impl Atlas {
    const SPACING: f32 = 2.0;

    pub fn new(glyphs: &mut [Glyph], font_size: f32) -> Self {
        let spacing = Self::SPACING.ceil() as usize;
        let (width, height) = Self::calculate_size(glyphs, font_size);
        let mut bitmap = vec![0; width * height];
        let mut horizontal_offset: usize = spacing;
        let mut vertical_offset: usize = spacing;
        let mut line_height: usize = 0;

        for glyph in glyphs.iter_mut() {
            let glyph_width = glyph.rect.width.ceil() as usize;
            let glyph_height = glyph.rect.height.ceil() as usize;

            if glyph_width == 0 && glyph_height == 0 {
                continue;
            }

            let mut glyph_horizontal_offset = horizontal_offset;
            horizontal_offset += glyph_width + 2 * spacing;

            if horizontal_offset > width {
                // go next line
                glyph_horizontal_offset = spacing;
                horizontal_offset = glyph_width + 3 * spacing;
                vertical_offset += line_height + spacing;
                line_height = 0;
            }

            if line_height < glyph_height {
                line_height = glyph_height;
                if vertical_offset + line_height > height {
                    panic!(
                        "The font atlas bitmap {}x{} size is not enough",
                        width, height
                    );
                }
            }

            let u0 = glyph_horizontal_offset as f32 / width as f32;
            let v0 = vertical_offset as f32 / height as f32;
            let u1 = (glyph_horizontal_offset + glyph_width) as f32 / width as f32;
            let v1 = (vertical_offset + glyph_height) as f32 / height as f32;

            glyph.uvs = [
                TexUV::new(u0, v0),
                TexUV::new(u1, v0),
                TexUV::new(u1, v1),
                TexUV::new(u0, v1),
            ];

            for row in 0..glyph_height {
                let offset = (vertical_offset + row) * width + glyph_horizontal_offset;
                for col in 0..glyph_width {
                    bitmap[offset + col] = glyph.bitmap[row * glyph_width + col];
                }
            }
        }

        Self {
            bitmap,
            width: width as u32,
            height: width as u32,
        }
    }

    fn calculate_size(glyphs: &[Glyph], font_size: f32) -> (usize, usize) {
        let reserve_factor = 1.2;
        let size = (reserve_factor
            * glyphs
                .iter()
                .map(|glyph| {
                    (glyph.rect.width + Self::SPACING) * (glyph.rect.height + Self::SPACING)
                })
                .sum::<f32>()
                .sqrt()
                .ceil()
            + font_size.ceil()) as usize;

        (size, size)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bitmap(&self) -> &[u8] {
        self.bitmap.as_slice()
    }
}

#[derive(Debug, Clone)]
pub struct LineMetrics {
    pub line_ascent: f32,
    pub line_descent: f32,
    pub line_gap: f32,
    pub line_size: f32,
}

pub struct Glyph {
    bitmap: Vec<u8>,
    uvs: [vertex::TexUV; 4],
    rect: Rect,
    advance_width: f32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Charset {
    Greek,
    Latin,
    Cyrillic,
}

impl Charset {
    pub fn range(&self) -> &'static [Range<u32>] {
        match self {
            Charset::Latin => &[
                0x0020..0x00FF,
                0x0100..0x017F,
                0x0180..0x024F,
                0x2C60..0x2C7F,
                0xA720..0xA7FF,
                0xAB30..0xAB6F,
            ],
            Charset::Cyrillic => &[0x0400..0x052F, 0x2DE0..0x2DFF, 0xA640..0xA69F],
            Charset::Greek => &[0x0370..0x03FF],
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub offset: Option<f32>,
    pub width: f32,
    pub height: f32,
}

// Note: the following code can be used for debug purposes
/*
    let charsets = [
        dotrix::ui::font::Charset::Latin,
        dotrix::ui::font::Charset::Cyrillic,
        dotrix::ui::font::Charset::Greek,
    ];
    let font_bytes = include_bytes!("../resources/fonts/Jura-Regular.ttf") as &[u8];
    let font = dotrix::ui::font::Font::from_bytes(28.0, &charsets, font_bytes);
    let atlas = font.atlas();
    let bitmap = atlas.bitmap();
    let width = atlas.width();
    let height = atlas.height();

    let mut img = image::ImageBuffer::new(width, height);
    for (pixel, alpha) in img.pixels_mut().zip(bitmap.iter()) {
        *pixel = image::Rgba([0, 0, 0, *alpha]);
    }
    match img.save("jura-atlas.png") {
        Ok(_) => panic!("Saved"),
        Err(err) => panic!("Error: {err}"),
    }
*/
