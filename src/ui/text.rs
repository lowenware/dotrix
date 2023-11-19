use crate::style;
use dotrix_types::Color;

pub struct Text {
    inner: String,
    style: style::Text,
}

impl Text {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            inner: value.into(),
            style: style::Text {
                font_size: 18.0,
                color: Color::white(),
                ..Default::default()
            },
        }
    }

    pub fn style(mut self, style: style::Text) -> Self {
        self.style = style;
        self
    }
}
