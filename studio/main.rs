mod camera;
mod states;
mod ui;

use dotrix::log;

/// This can be used in tasks context
pub struct Studio {}

impl Studio {
    fn new() -> Self {
        Self {}
    }
}

impl dotrix::Application for Studio {
    fn configure(&self, settings: &mut dotrix::Settings) {
        settings.title = String::from("Dotrix Studio");
    }

    fn init(&self, manager: &mut dotrix::Manager) {
        manager.schedule(states::Startup::new());
    }
}

fn main() {
    let log = dotrix::Log {
        targets: vec![
            (String::from("wgpu"), log::LevelFilter::Warn),
            (String::from("dotrix"), log::LevelFilter::Debug),
            (String::from(""), log::LevelFilter::Debug),
        ],
        ..Default::default()
    };
    log::subscribe(log);

    let studio = Studio::new();

    let assets = dotrix::assets::Extension {
        root: std::path::PathBuf::from("./resources"),
        init: |assets| {
            assets.install(dotrix::image::Loader::default());
            assets.install(dotrix::shader::Loader::default());
        },
        hot_reload: true,
    };

    let camera_control_task = camera::ControlTask::new();

    let pbr = dotrix::pbr::Extension::default();

    let ui = dotrix::ui::Extension::default();

    let ui_task = ui::UiTask::default();

    dotrix::run(studio, |core| {
        core.extend_with(assets);
        core.extend_with(pbr);
        core.extend_with(ui);
        core.schedule(camera_control_task);
        core.schedule(ui_task);
    });
}
