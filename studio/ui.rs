#[derive(Default)]
pub struct UiTask {}

impl dotrix::Task for UiTask {
    type Context = (dotrix::Any<dotrix::Frame>,);
    type Output = dotrix::ui::Overlay;
    fn run(&mut self, (frame,): Self::Context) -> Self::Output {
        dotrix::ui::Overlay {
            rect: dotrix::ui::Rect {
                horizontal: 24.0,
                vertical: 24.0,
                width: 280.0,
                height: 400.0,
            },
            view: dotrix::ui::View::default(),
        }
    }
}
