use super::super::label::{layout_label, measure_text, text_height};
use super::super::layout::LayoutRect;
use super::super::{ButtonResponse, ButtonStyle, Ui, UiDrawCommand};

impl Ui {
    pub fn button(&mut self, style: ButtonStyle, label: &str) -> ButtonResponse {
        let (text_w, text_h) = {
            let font = self
                .assets
                .get(style.text_style.font)
                .expect("button font must exist in Assets");
            (measure_text(font, label), text_height(font, label))
        };
        let width = text_w + style.padding.left + style.padding.right;
        let height = text_h + style.padding.top + style.padding.bottom;

        let rect = self.allocate_vertical(LayoutRect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        });

        let region_index = self.push_hit_region(rect, super::super::HitKind::Button);
        let hovered = self.ui_state.hovered_region == Some(region_index);
        let pressed = self.ui_state.pressed_region == Some(region_index);
        let clicked = self.ui_state.clicked_region == Some(region_index);

        let (bg, fg) = if pressed {
            (style.pressed_background, style.pressed_text_color)
        } else if hovered {
            (style.hover_background, style.hover_text_color)
        } else {
            (style.background, style.text_style.color)
        };

        self.commands.push(UiDrawCommand::Rect {
            rect,
            color: bg,
            corner_radius: style.corner_radius,
        });

        let text_origin = LayoutRect {
            x: rect.x + style.padding.left,
            y: rect.y + style.padding.top,
            width: text_w,
            height: text_h,
        };
        let font = self
            .assets
            .get(style.text_style.font)
            .expect("button font must exist in Assets");
        layout_label(
            &mut self.commands,
            style.text_style.font,
            font,
            fg,
            text_origin,
            label,
        );

        ButtonResponse {
            hovered,
            pressed,
            clicked,
            region_index,
        }
    }
}

impl ButtonStyle {
    pub fn new(text_style: super::super::TextStyle) -> Self {
        Self {
            text_style,
            ..Default::default()
        }
    }
}
