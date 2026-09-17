use crate::loaders::Assets;
use crate::{Color, Extent2D, Id, Ref};

use super::font::Font;
use super::input::{HitKind, HitRegion, UiState};
use super::label::{layout_label, measure_text, text_height};
use super::layout::{resolve_view, LayoutRect};
use super::view::{PositionConstraint, SizeConstraint, View};

/// Per-frame UI draw list produced by an app build task.
#[derive(Debug, Default, Clone)]
pub struct Overlay {
    pub commands: Vec<UiDrawCommand>,
    pub hit_regions: Vec<HitRegion>,
}

impl Overlay {
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// A single UI primitive in the draw list.
#[derive(Debug, Clone, Copy)]
pub enum UiDrawCommand {
    Rect {
        rect: LayoutRect,
        color: Color<f32>,
        corner_radius: f32,
    },
    Glyph {
        rect: LayoutRect,
        color: Color<f32>,
        font: Id<Font>,
        uv_min: [f32; 2],
        uv_max: [f32; 2],
    },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Spacing {
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub font: Id<Font>,
    pub color: Color<f32>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: Id::null(),
            color: Color::white(),
        }
    }
}

impl TextStyle {
    pub fn font(mut self, font: Id<Font>) -> Self {
        self.font = font;
        self
    }

    pub fn color(mut self, color: Color<f32>) -> Self {
        self.color = color;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PanelStyle {
    pub background: Color<f32>,
    pub corner_radius: f32,
    pub padding: Spacing,
    pub min_size: LayoutRect,
    pub interactive: bool,
}

impl Default for PanelStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(0.0, 0.0, 0.0, 0.5),
            corner_radius: 8.0,
            padding: Spacing::uniform(8.0),
            min_size: LayoutRect::default(),
            interactive: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonStyle {
    pub text_style: TextStyle,
    pub background: Color<f32>,
    pub hover_background: Color<f32>,
    pub pressed_background: Color<f32>,
    pub hover_text_color: Color<f32>,
    pub pressed_text_color: Color<f32>,
    pub corner_radius: f32,
    pub padding: Spacing,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            text_style: TextStyle::default(),
            background: Color::rgba(0.2, 0.2, 0.2, 0.9),
            hover_background: Color::rgba(0.3, 0.3, 0.3, 0.9),
            pressed_background: Color::rgba(0.15, 0.15, 0.15, 0.9),
            hover_text_color: Color::white(),
            pressed_text_color: Color::white(),
            corner_radius: 6.0,
            padding: Spacing::uniform(8.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonResponse {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub region_index: usize,
}

struct LayoutScope {
    rect: LayoutRect,
    cursor_y: f32,
    region_index: Option<usize>,
}

/// Immediate-mode UI builder.
pub struct Ui {
    resolution: Extent2D,
    pub(crate) assets: Ref<Assets>,
    pub(crate) ui_state: Ref<UiState>,
    pub(crate) commands: Vec<UiDrawCommand>,
    hit_regions: Vec<HitRegion>,
    next_region_index: usize,
    scopes: Vec<LayoutScope>,
}

impl Ui {
    pub fn new(resolution: Extent2D, assets: Ref<Assets>, ui_state: Ref<UiState>) -> Self {
        let root = LayoutRect {
            x: 0.0,
            y: 0.0,
            width: resolution.width as f32,
            height: resolution.height as f32,
        };
        Self {
            resolution,
            assets,
            ui_state,
            commands: Vec::new(),
            hit_regions: Vec::new(),
            next_region_index: 0,
            scopes: vec![LayoutScope {
                rect: root,
                cursor_y: root.y,
                region_index: None,
            }],
        }
    }

    pub fn finish(self) -> Overlay {
        Overlay {
            commands: self.commands,
            hit_regions: self.hit_regions,
        }
    }

    pub fn label(&mut self, style: TextStyle, text: impl AsRef<str>) {
        let text = text.as_ref();
        let (text_w, text_h) = {
            let font = self
                .assets
                .get(style.font)
                .expect("label font must exist in Assets");
            (measure_text(font, text), text_height(font, text))
        };
        let rect = self.allocate_vertical(LayoutRect {
            x: 0.0,
            y: 0.0,
            width: text_w,
            height: text_h,
        });
        let font = self
            .assets
            .get(style.font)
            .expect("label font must exist in Assets");
        layout_label(
            &mut self.commands,
            style.font,
            font,
            style.color,
            rect,
            text,
        );
    }

    pub fn spacer(&mut self, size: f32) {
        self.allocate_vertical(LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: size,
        });
    }

    pub fn view(&mut self, view: View, content: impl FnOnce(&mut Ui)) {
        let parent = self.current_scope().rect;
        let rect = resolve_view(&view, parent, self.resolution);
        self.push_scope(rect, None);
        content(self);
        self.pop_scope();
    }

    pub(crate) fn push_hit_region(&mut self, rect: LayoutRect, kind: HitKind) -> usize {
        let index = self.next_region_index;
        self.next_region_index += 1;
        self.hit_regions.push(HitRegion { index, rect, kind });
        index
    }

    pub(crate) fn next_rect(&mut self, min_size: LayoutRect) -> LayoutRect {
        if min_size.width > 0.0 && min_size.height > 0.0 {
            self.allocate_vertical(min_size)
        } else {
            let scope = self.current_scope();
            let remaining = (scope.rect.y + scope.rect.height - scope.cursor_y).max(0.0);
            LayoutRect {
                x: scope.rect.x,
                y: scope.cursor_y,
                width: scope.rect.width,
                height: remaining,
            }
        }
    }

    pub(crate) fn allocate_vertical(&mut self, size: LayoutRect) -> LayoutRect {
        let scope = self.scopes.last_mut().expect("layout scope");
        let rect = LayoutRect {
            x: scope.rect.x,
            y: scope.cursor_y,
            width: size.width.max(scope.rect.width),
            height: size.height,
        };
        scope.cursor_y += size.height;
        rect
    }

    fn current_scope(&self) -> &LayoutScope {
        self.scopes.last().expect("layout scope")
    }

    pub(crate) fn push_scope(&mut self, rect: LayoutRect, region_index: Option<usize>) {
        self.scopes.push(LayoutScope {
            rect,
            cursor_y: rect.y,
            region_index,
        });
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}

/// Convenience view for a top-left anchored panel.
pub fn top_left_panel_view(width: f32, height: f32, margin: f32) -> View {
    View {
        x: PositionConstraint::Pixel(margin),
        y: PositionConstraint::Pixel(margin),
        width: SizeConstraint::Pixel(width),
        height: SizeConstraint::Pixel(height),
    }
}
