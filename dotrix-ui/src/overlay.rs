use crate::composer::Composer;
use crate::style::Spacing;
use crate::View;

pub struct Overlay {
    pub rect: Rect,
    pub view: View,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Rect {
    pub horizontal: f32,
    pub vertical: f32,
    pub width: f32,
    pub height: f32,
}

impl Overlay {
    pub fn compose(&self, composer: &mut Composer) {
        use crate::composer::Compose;
        self.view.compose(&self.rect, composer);
        composer.widgets.reverse();
    }
}

impl Rect {
    pub fn inner(&self, spacing: &Spacing) -> Rect {
        let width = self.width - spacing.left - spacing.right;
        let height = self.height - spacing.top - spacing.bottom;
        let horizontal = if self.horizontal >= 0.0 {
            self.horizontal + spacing.left
        } else {
            self.horizontal - spacing.right
        };
        let vertical = if self.vertical >= 0.0 {
            self.vertical + spacing.top
        } else {
            self.vertical - spacing.bottom
        };

        Rect {
            horizontal,
            vertical,
            width: width.clamp(0.0, self.width),
            height: height.clamp(0.0, self.height),
        }
    }
}
