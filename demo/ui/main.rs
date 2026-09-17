use dotrix::ui::{
    top_left_panel_view, ButtonStyle, Charset, PanelStyle, ProcessUiInput, RenderOverlay, Spacing,
    TextStyle, Ui, UiState,
};
use dotrix::{log, Font};

struct UiDemoApp;

impl Default for UiDemoApp {
    fn default() -> Self {
        Self
    }
}

struct BuildDemoUi {
    ui_font: dotrix::Id<Font>,
    click_count: u32,
}

impl Default for BuildDemoUi {
    fn default() -> Self {
        Self {
            ui_font: dotrix::Id::null(),
            click_count: 0,
        }
    }
}

impl dotrix::Task for BuildDemoUi {
    type Context = (
        dotrix::Any<dotrix::Frame>,
        dotrix::Ref<dotrix::Assets>,
        dotrix::Ref<UiState>,
    );
    type Output = dotrix::ui::Overlay;

    fn run(&mut self, (frame, assets, ui_state): Self::Context) -> Self::Output {
        if self.ui_font.is_null() {
            self.ui_font = assets
                .find("ui-font-regular-24")
                .expect("ui font must be loaded");
        }

        let mut ui = Ui::new(frame.resolution, assets, ui_state);

        ui.view(top_left_panel_view(280.0, 190.0, 16.0), |ui| {
            ui.panel(
                PanelStyle {
                    background: dotrix::Color::rgba(0.08, 0.08, 0.12, 0.85),
                    corner_radius: 10.0,
                    padding: Spacing::uniform(12.0),
                    ..Default::default()
                },
                |ui| {
                    ui.label(
                        TextStyle::default()
                            .font(self.ui_font)
                            .color(dotrix::Color::white()),
                        "Dotrix UI Demo",
                    );
                    ui.spacer(8.0);
                    ui.label(
                        TextStyle::default()
                            .font(self.ui_font)
                            .color(dotrix::Color::rgba(0.8, 0.8, 0.9, 1.0)),
                        format!("FPS: {:.0}", frame.fps),
                    );
                    ui.spacer(12.0);
                    let response = ui.button(
                        ButtonStyle {
                            text_style: TextStyle::default()
                                .font(self.ui_font)
                                .color(dotrix::Color::white()),
                            background: dotrix::Color::rgba(0.22, 0.24, 0.32, 0.95),
                            hover_background: dotrix::Color::rgba(0.32, 0.42, 0.62, 0.95),
                            pressed_background: dotrix::Color::rgba(0.16, 0.20, 0.30, 0.95),
                            hover_text_color: dotrix::Color::white(),
                            pressed_text_color: dotrix::Color::white(),
                            corner_radius: 6.0,
                            padding: Spacing::uniform(8.0),
                        },
                        "Click me",
                    );
                    if response.clicked {
                        self.click_count += 1;
                    }
                    ui.spacer(8.0);
                    ui.label(
                        TextStyle::default()
                            .font(self.ui_font)
                            .color(dotrix::Color::rgba(0.7, 0.7, 0.8, 1.0)),
                        format!("Clicks: {}", self.click_count),
                    );
                },
            );
        });

        ui.finish()
    }
}

impl dotrix::Application for UiDemoApp {
    fn app_name(&self) -> &str {
        "Dotrix UI Demo"
    }

    fn device_type_request(&self) -> Option<dotrix::DeviceType> {
        Some(dotrix::DeviceType::Integrated)
    }

    fn startup(
        self,
        scheduler: &dotrix::tasks::Scheduler,
        display: &mut dotrix::graphics::Display,
    ) {
        log::info!("Starting Dotrix UI demo");

        let mut assets = dotrix::Assets::default();
        assets.set(
            Font::from_file(
                "ui-font-regular-24",
                "resources/fonts/Jura-Regular.ttf",
                24.0,
                &Charset::latin_extended(),
            )
            .expect("failed to load Inter font"),
        );

        scheduler.add_context(assets);
        scheduler.add_context(UiState::default());

        scheduler.add_task(BuildDemoUi::default());
        scheduler.add_task(ProcessUiInput::default());

        let overlay_renderer = RenderOverlay::setup().create(display);
        scheduler.add_task(overlay_renderer);
    }
}

fn main() {
    dotrix::Log::default()
        .level("dotrix", log::LevelFilter::Debug)
        .level("*", log::LevelFilter::Info)
        .subscribe();

    dotrix::run(UiDemoApp::default());
}
