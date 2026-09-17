use crate::Extent2D;

use super::view::{PositionConstraint, SizeConstraint, View};

/// Pixel-space axis-aligned rectangle.
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutRect {
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.width && py < self.y + self.height
    }

    pub fn inner(&self, padding: &super::Spacing) -> Self {
        Self {
            x: self.x + padding.left,
            y: self.y + padding.top,
            width: (self.width - padding.left - padding.right).max(0.0),
            height: (self.height - padding.top - padding.bottom).max(0.0),
        }
    }
}

pub fn resolve_view(view: &View, parent: LayoutRect, resolution: Extent2D) -> LayoutRect {
    let screen_w = resolution.width as f32;
    let screen_h = resolution.height as f32;
    let width = resolve_width(&view.width, parent.width, screen_w);
    let height = resolve_height(&view.height, parent.height, screen_h);

    let x = resolve_x(&view.x, parent, width, screen_w);
    let y = resolve_y(&view.y, parent, height, screen_h);

    LayoutRect {
        x,
        y,
        width,
        height,
    }
}

fn resolve_width(constraint: &SizeConstraint, parent_width: f32, screen_width: f32) -> f32 {
    let value = match constraint {
        SizeConstraint::Pixel(value) => *value,
        SizeConstraint::Percent(fraction) => parent_width * fraction,
    };
    value.min(screen_width).max(0.0)
}

fn resolve_height(constraint: &SizeConstraint, parent_height: f32, screen_height: f32) -> f32 {
    let value = match constraint {
        SizeConstraint::Pixel(value) => *value,
        SizeConstraint::Percent(fraction) => parent_height * fraction,
    };
    value.min(screen_height).max(0.0)
}

fn resolve_x(
    constraint: &PositionConstraint,
    parent: LayoutRect,
    width: f32,
    screen_width: f32,
) -> f32 {
    match constraint {
        PositionConstraint::Pixel(offset) => parent.x + offset,
        PositionConstraint::Percent(fraction) => parent.x + parent.width * fraction,
        PositionConstraint::Center => parent.x + (parent.width - width) * 0.5,
    }
    .clamp(0.0, screen_width.max(width))
}

fn resolve_y(
    constraint: &PositionConstraint,
    parent: LayoutRect,
    height: f32,
    screen_height: f32,
) -> f32 {
    match constraint {
        PositionConstraint::Pixel(offset) => parent.y + offset,
        PositionConstraint::Percent(fraction) => parent.y + parent.height * fraction,
        PositionConstraint::Center => parent.y + (parent.height - height) * 0.5,
    }
    .clamp(0.0, screen_height.max(height))
}
