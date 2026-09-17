use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::ops::Range;
use std::path::Path;

use crate::loaders::Asset;
use crate::log;

/// Baked font binary format version byte.
pub const BAKED_FONT_VERSION: u8 = 1;

/// Unicode code-point ranges for font rasterization.
#[derive(Debug, Clone)]
pub struct Charset {
    ranges: Vec<Range<u32>>,
}

impl Charset {
    /// ASCII + Latin-1 Supplement + Latin Extended-A (European diacritics).
    pub fn latin_extended() -> Self {
        Self {
            ranges: vec![0x0020..0x0080, 0x00A0..0x0100, 0x0100..0x0180],
        }
    }

    pub fn codepoints(&self) -> Vec<char> {
        let mut chars = Vec::new();
        for range in &self.ranges {
            for code in range.clone() {
                if let Some(ch) = char::from_u32(code) {
                    chars.push(ch);
                }
            }
        }
        chars
    }
}

/// Per-glyph metrics and atlas UVs.
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub width: f32,
    pub height: f32,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

/// Line metrics for text layout.
#[derive(Debug, Clone, Copy)]
pub struct LineMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub gap: f32,
    pub line_size: f32,
}

/// Font asset — one pixel size per asset instance.
pub struct Font {
    name: String,
    size_px: f32,
    atlas: Vec<u8>,
    atlas_width: u32,
    atlas_height: u32,
    line_metrics: LineMetrics,
    glyphs: HashMap<char, GlyphMetrics>,
}

impl Asset for Font {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Font {
    pub fn size_px(&self) -> f32 {
        self.size_px
    }

    pub fn line_metrics(&self) -> &LineMetrics {
        &self.line_metrics
    }

    pub fn atlas_width(&self) -> u32 {
        self.atlas_width
    }

    pub fn atlas_height(&self) -> u32 {
        self.atlas_height
    }

    pub fn atlas_pixels(&self) -> &[u8] {
        &self.atlas
    }

    pub fn glyph(&self, ch: char) -> Option<&GlyphMetrics> {
        self.glyphs.get(&ch)
    }

    /// Load a font from a baked binary produced by [`bake_ttf_to_bytes`] or mythstic `build.rs`.
    pub fn from_baked_bin(name: impl Into<String>, bytes: &[u8]) -> Result<Self, String> {
        let mut cursor = Cursor::new(bytes);

        let mut version = [0u8; 1];
        cursor.read_exact(&mut version).map_err(|e| e.to_string())?;
        if version[0] != BAKED_FONT_VERSION {
            return Err(format!(
                "unsupported baked font version {} (expected {})",
                version[0], BAKED_FONT_VERSION
            ));
        }

        let size_px = read_f32(&mut cursor)?;
        let atlas_width = read_u32(&mut cursor)?;
        let atlas_height = read_u32(&mut cursor)?;
        let ascent = read_f32(&mut cursor)?;
        let descent = read_f32(&mut cursor)?;
        let gap = read_f32(&mut cursor)?;
        let line_size = read_f32(&mut cursor)?;
        let glyph_count = read_u32(&mut cursor)? as usize;

        let atlas_len = (atlas_width * atlas_height) as usize;
        let mut atlas = vec![0u8; atlas_len];
        cursor.read_exact(&mut atlas).map_err(|e| e.to_string())?;

        let mut glyphs = HashMap::with_capacity(glyph_count);
        for _ in 0..glyph_count {
            let codepoint = read_u32(&mut cursor)?;
            let ch = char::from_u32(codepoint)
                .ok_or_else(|| format!("invalid codepoint {codepoint}"))?;
            let metrics = GlyphMetrics {
                advance: read_f32(&mut cursor)?,
                bearing_x: read_f32(&mut cursor)?,
                bearing_y: read_f32(&mut cursor)?,
                width: read_f32(&mut cursor)?,
                height: read_f32(&mut cursor)?,
                uv_min: [read_f32(&mut cursor)?, read_f32(&mut cursor)?],
                uv_max: [read_f32(&mut cursor)?, read_f32(&mut cursor)?],
            };
            glyphs.insert(ch, metrics);
        }

        Ok(Self {
            name: name.into(),
            size_px,
            atlas,
            atlas_width,
            atlas_height,
            line_metrics: LineMetrics {
                ascent,
                descent,
                gap,
                line_size,
            },
            glyphs,
        })
    }

    /// Rasterize a TTF/OTF file at runtime (requires `fontdue` feature).
    #[cfg(feature = "fontdue")]
    pub fn from_file(
        name: impl Into<String>,
        path: impl AsRef<Path>,
        size_px: f32,
        charset: &Charset,
    ) -> Result<Self, String> {
        let bytes = std::fs::read(path.as_ref()).map_err(|e| e.to_string())?;
        Self::from_bytes(name, &bytes, size_px, charset)
    }

    /// Rasterize font bytes at runtime (requires `fontdue` feature).
    #[cfg(feature = "fontdue")]
    pub fn from_bytes(
        name: impl Into<String>,
        bytes: &[u8],
        size_px: f32,
        charset: &Charset,
    ) -> Result<Self, String> {
        let baked = bake_ttf_to_bytes(bytes, size_px, charset)?;
        Self::from_baked_bin(name, &baked)
    }
}

/// Bake a TTF/OTF into the on-disk `.bin` layout (requires `fontdue` feature).
#[cfg(feature = "fontdue")]
pub fn bake_ttf_to_bytes(bytes: &[u8], size_px: f32, charset: &Charset) -> Result<Vec<u8>, String> {
    let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|e| e.to_string())?;

    let line_metrics = font
        .horizontal_line_metrics(size_px)
        .map(|m| LineMetrics {
            ascent: m.ascent,
            descent: m.descent,
            gap: m.line_gap,
            line_size: m.new_line_size,
        })
        .ok_or_else(|| "font missing horizontal line metrics".to_string())?;

    let codepoints = charset.codepoints();
    let mut glyph_data: Vec<(char, fontdue::Metrics, Vec<u8>)> =
        Vec::with_capacity(codepoints.len());

    for ch in codepoints {
        let (metrics, bitmap) = font.rasterize(ch, size_px);
        glyph_data.push((ch, metrics, bitmap));
    }

    let spacing = 2usize;
    let (atlas_width, atlas_height) = estimate_atlas_size(&glyph_data, spacing);
    let mut atlas = vec![0u8; atlas_width * atlas_height];
    let mut glyphs = HashMap::with_capacity(glyph_data.len());

    let mut x = spacing;
    let mut y = spacing;
    let mut row_height = 0usize;

    for (ch, metrics, bitmap) in glyph_data {
        let gw = metrics.width.max(0) as usize;
        let gh = metrics.height.max(0) as usize;

        if gw > 0 && gh > 0 {
            if x + gw + spacing > atlas_width {
                x = spacing;
                y += row_height + spacing;
                row_height = 0;
            }
            if y + gh + spacing > atlas_height {
                return Err("font atlas overflow during bake".to_string());
            }

            for row in 0..gh {
                let dst = (y + row) * atlas_width + x;
                let src = row * gw;
                atlas[dst..dst + gw].copy_from_slice(&bitmap[src..src + gw]);
            }

            let u0 = x as f32 / atlas_width as f32;
            let v0 = y as f32 / atlas_height as f32;
            let u1 = (x + gw) as f32 / atlas_width as f32;
            let v1 = (y + gh) as f32 / atlas_height as f32;

            glyphs.insert(
                ch,
                GlyphMetrics {
                    advance: metrics.advance_width,
                    bearing_x: metrics.xmin as f32,
                    bearing_y: metrics.ymin as f32,
                    width: gw as f32,
                    height: gh as f32,
                    uv_min: [u0, v0],
                    uv_max: [u1, v1],
                },
            );

            x += gw + spacing;
            row_height = row_height.max(gh);
        } else {
            glyphs.insert(
                ch,
                GlyphMetrics {
                    advance: metrics.advance_width,
                    bearing_x: metrics.xmin as f32,
                    bearing_y: metrics.ymin as f32,
                    width: 0.0,
                    height: 0.0,
                    uv_min: [0.0, 0.0],
                    uv_max: [0.0, 0.0],
                },
            );
        }
    }

    write_baked_bytes(
        size_px,
        atlas_width as u32,
        atlas_height as u32,
        &line_metrics,
        &atlas,
        &glyphs,
    )
}

#[cfg(feature = "fontdue")]
fn estimate_atlas_size(
    glyphs: &[(char, fontdue::Metrics, Vec<u8>)],
    spacing: usize,
) -> (usize, usize) {
    let area: f32 = glyphs
        .iter()
        .map(|(_, m, _)| {
            (m.width.max(1) as f32 + spacing as f32) * (m.height.max(1) as f32 + spacing as f32)
        })
        .sum();
    let side = (area.sqrt() * 1.3).ceil() as usize;
    (side.max(64), side.max(64))
}

pub(crate) fn write_baked_bytes(
    size_px: f32,
    atlas_width: u32,
    atlas_height: u32,
    line_metrics: &LineMetrics,
    atlas: &[u8],
    glyphs: &HashMap<char, GlyphMetrics>,
) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    out.write_all(&[BAKED_FONT_VERSION])
        .map_err(|e| e.to_string())?;
    write_f32(&mut out, size_px)?;
    write_u32(&mut out, atlas_width)?;
    write_u32(&mut out, atlas_height)?;
    write_f32(&mut out, line_metrics.ascent)?;
    write_f32(&mut out, line_metrics.descent)?;
    write_f32(&mut out, line_metrics.gap)?;
    write_f32(&mut out, line_metrics.line_size)?;
    write_u32(&mut out, glyphs.len() as u32)?;
    out.write_all(atlas).map_err(|e| e.to_string())?;

    let mut sorted: Vec<_> = glyphs.iter().collect();
    sorted.sort_by_key(|(ch, _)| **ch as u32);
    for (ch, metrics) in sorted {
        write_u32(&mut out, *ch as u32)?;
        write_f32(&mut out, metrics.advance)?;
        write_f32(&mut out, metrics.bearing_x)?;
        write_f32(&mut out, metrics.bearing_y)?;
        write_f32(&mut out, metrics.width)?;
        write_f32(&mut out, metrics.height)?;
        write_f32(&mut out, metrics.uv_min[0])?;
        write_f32(&mut out, metrics.uv_min[1])?;
        write_f32(&mut out, metrics.uv_max[0])?;
        write_f32(&mut out, metrics.uv_max[1])?;
    }

    Ok(out)
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, String> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(u32::from_le_bytes(buf))
}

fn read_f32(cursor: &mut Cursor<&[u8]>) -> Result<f32, String> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(f32::from_le_bytes(buf))
}

fn write_u32(out: &mut Vec<u8>, value: u32) -> Result<(), String> {
    out.write_all(&value.to_le_bytes())
        .map_err(|e| e.to_string())
}

fn write_f32(out: &mut Vec<u8>, value: f32) -> Result<(), String> {
    out.write_all(&value.to_le_bytes())
        .map_err(|e| e.to_string())
}

/// Bake a font file to a `.bin` on disk (requires `fontdue` feature).
#[cfg(feature = "fontdue")]
pub fn bake_ttf_file_to_path(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    size_px: f32,
    charset: &Charset,
) -> Result<(), String> {
    let bytes = std::fs::read(input.as_ref()).map_err(|e| e.to_string())?;
    let baked = bake_ttf_to_bytes(&bytes, size_px, charset)?;
    std::fs::write(output.as_ref(), &baked).map_err(|e| e.to_string())?;
    log::info!(
        "baked font `{}` -> `{}` ({} bytes)",
        input.as_ref().display(),
        output.as_ref().display(),
        baked.len()
    );
    Ok(())
}
