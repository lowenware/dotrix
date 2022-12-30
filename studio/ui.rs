use dotrix::ui::style;
use dotrix::Color;

#[derive(Default)]
pub struct UiTask {}

impl dotrix::Task for UiTask {
    type Context = (dotrix::Any<dotrix::Frame>,);
    type Output = dotrix::ui::Overlay;
    fn run(&mut self, (frame,): Self::Context) -> Self::Output {
        let mut root = dotrix::ui::View::new(dotrix::ui::Style {
            background: Some(style::Background::from_color(Color::white())),
            padding: style::Spacing {
                top: 2.0,
                left: 3.0,
                right: 15.0,
                bottom: 4.0,
            },
            ..Default::default()
        });

        let inner = dotrix::ui::View::new(dotrix::ui::Style {
            background: Some(style::Background::from_color(Color::red())),
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
