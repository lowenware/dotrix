use dotrix::ui::style;
use dotrix::{Color, Id, TexUV};

#[derive(Default)]
pub struct UiTask {}

impl dotrix::Task for UiTask {
    type Context = (dotrix::Any<dotrix::Frame>,);
    type Output = dotrix::ui::Overlay;
    fn run(&mut self, (frame,): Self::Context) -> Self::Output {
        let mut root = dotrix::ui::View::new(dotrix::ui::Style {
            background: Some(style::Background::from_color(Color::red())),
            padding: style::Spacing {
                top: 2.0,
                left: 3.0,
                right: 15.0,
                bottom: 4.0,
            },
            ..Default::default()
        });

        let inner = dotrix::ui::View::new(dotrix::ui::Style {
            background: Some(style::Background {
                color: style::Corners {
                    top_left: Color::white(),
                    top_right: Color::white(),
                    bottom_right: Color::white(),
                    bottom_left: Color::white(),
                },
                uvs: style::Corners {
                    top_left: TexUV::new(0.0, 0.0),
                    top_right: TexUV::new(1.0, 0.0),
                    bottom_right: TexUV::new(1.0, 1.0),
                    bottom_left: TexUV::new(0.0, 1.0),
                },
                texture: Id::default(),
            }),
            ..Default::default()
        });

        root.append(inner);

        dotrix::ui::Overlay {
            rect: dotrix::ui::Rect {
                horizontal: 24.0,
                vertical: 24.0,
                width: 280.0,
                height: 400.0,
            },
            view: root,
        }
    }
}
