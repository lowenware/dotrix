use crate::{Color, Id};

use super::font::Font;
use super::layout::LayoutRect;
use super::UiDrawCommand;

/// Ink bounds of a single-line string relative to its baseline.
#[derive(Debug, Clone, Copy)]
pub struct TextExtents {
    pub width: f32,
    /// Distance from the baseline up to the highest ink pixel.
    pub above_baseline: f32,
    /// Distance from the baseline down to the lowest ink pixel.
    pub below_baseline: f32,
}

impl TextExtents {
    pub fn ink_height(&self) -> f32 {
        self.above_baseline + self.below_baseline
    }
}

/// Measure ink bounds for a single-line string.
pub fn measure_text_extents(font: &Font, text: &str) -> TextExtents {
    let mut width = 0.0;
    let mut above_baseline = 0.0f32;
    let mut below_baseline = 0.0f32;

    for ch in text.chars() {
        if let Some(glyph) = font.glyph(ch) {
            // fontdue ymin is the bitmap bottom-edge offset in y-up space.
            let top = -glyph.bearing_y - glyph.height;
            let bottom = -glyph.bearing_y;
            above_baseline = above_baseline.max(-top);
            below_baseline = below_baseline.max(bottom);
            width += glyph.advance;
        }
    }

    if width == 0.0 {
        let metrics = font.line_metrics();
        above_baseline = metrics.ascent;
        below_baseline = -metrics.descent;
    }

    TextExtents {
        width,
        above_baseline,
        below_baseline,
    }
}

/// Measure the pixel width of a single-line string.
pub fn measure_text(font: &Font, text: &str) -> f32 {
    measure_text_extents(font, text).width
}

/// Measure the pixel height of a single-line string's ink.
pub fn text_height(font: &Font, text: &str) -> f32 {
    measure_text_extents(font, text).ink_height()
}

/// Emit glyph draw commands for a single-line label.
pub fn layout_label(
    commands: &mut Vec<UiDrawCommand>,
    font_id: Id<Font>,
    font: &Font,
    color: Color<f32>,
    origin: LayoutRect,
    text: &str,
) {
    let extents = measure_text_extents(font, text);
    // Bottom-align ink within the allocated rect.
    let baseline_y = origin.y + origin.height - extents.below_baseline;
    let mut cursor_x = origin.x;

    for ch in text.chars() {
        if let Some(glyph) = font.glyph(ch) {
            if glyph.width > 0.0 && glyph.height > 0.0 {
                let x = cursor_x + glyph.bearing_x;
                let y = baseline_y - glyph.bearing_y - glyph.height;
                commands.push(UiDrawCommand::Glyph {
                    rect: LayoutRect {
                        x,
                        y,
                        width: glyph.width,
                        height: glyph.height,
                    },
                    color,
                    font: font_id,
                    uv_min: glyph.uv_min,
                    uv_max: glyph.uv_max,
                });
            }
            cursor_x += glyph.advance;
        }
    }
}
