use dotrix::log;
use dotrix::ui::style;
use dotrix::ui::{Context, Rect, Style, Text, View};
use dotrix::{Color, Id, TexUV};

#[derive(Default)]
pub struct UiTask {
    ctx: Context,
}

impl dotrix::Task for UiTask {
    type Context = (dotrix::Any<dotrix::Input>, dotrix::Any<dotrix::Frame>);
    type Output = dotrix::ui::Overlay;
    fn run(&mut self, (input, frame): Self::Context) -> Self::Output {
        let rect = Rect {
            width: 200.0,
            height: 200.0,
            horizontal: 20.0,
            vertical: 20.0,
        };

        let mut my_string = String::with_capacity(64);

        let root_style = Style {
            background: Some(style::Background::from_color(Color::blue())),
            padding: style::Spacing::from(3.0),
            ..Default::default()
        };
        let child_style = Style {
            background: Some(style::Background::from_color(Color::white())),
            padding: style::Spacing::from(16.0),
            ..Default::default()
        };
        self.ctx.update(&input, &frame);

        log::warn!("Begin overlay");
        self.ctx.overlay(rect, |ui| {
            log::warn!("First recursion: sign in");
            ui.view(Some("root"), &root_style, |ui| {
                log::warn!("Second recursion: sign in");
                ui.view(None, &child_style, |_ui| {
                    log::warn!("Third recursion");
                });
                log::warn!("Second recursion: sign out");
                // ui.label(text_style, "Some Text");
                // ui.input(input_style, &mut my_string);
            });
            log::warn!("First recursion: sign out");
        })

        /*
        let root = dotrix::ui::View::new(dotrix::ui::Style {
            background: Some(style::Background::from_color(Color::red())),
            padding: style::Spacing {
                top: 2.0,
                left: 3.0,
                right: 15.0,
                bottom: 4.0,
            },
            ..Default::default()
        })
        .append(
            dotrix::ui::View::new(dotrix::ui::Style {
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
            })
            .append(dotrix::ui::Text::new("My first text").style(style::Text::from(Color::red()))),
        );

        dotrix::ui::Overlay {
            rect: dotrix::ui::Rect {
                horizontal: 24.0,
                vertical: 24.0,
                width: 251.0,
                height: 251.0,
            },
            view: root,
        }
             */
    }
}
