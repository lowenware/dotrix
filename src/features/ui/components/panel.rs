use super::super::layout::LayoutRect;
use super::super::{PanelStyle, Ui, UiDrawCommand};

impl Ui {
    pub fn panel(&mut self, style: PanelStyle, content: impl FnOnce(&mut Ui)) {
        let rect = self.next_rect(style.min_size);
        let region_index = if style.interactive {
            Some(self.push_hit_region(rect, super::super::HitKind::Panel))
        } else {
            None
        };

        self.commands.push(UiDrawCommand::Rect {
            rect,
            color: style.background,
            corner_radius: style.corner_radius,
        });

        let inner = rect.inner(&style.padding);
        self.push_scope(inner, region_index);
        content(self);
        self.pop_scope();
    }
}

impl PanelStyle {
    pub fn sized(width: f32, height: f32) -> Self {
        Self {
            min_size: LayoutRect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            ..Default::default()
        }
    }
}
