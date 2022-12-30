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
        // settings.fps_limit = Some(30.0);
    }

    fn init(&self, manager: &dotrix::Manager) {
        manager.schedule(states::Startup::new());
        manager.schedule(camera::ControlTask::new());
        manager.schedule(ui::UiTask::default());
    }
}

fn main() {
    let log = dotrix::Log {
        targets: vec![
            (String::from("naga"), log::LevelFilter::Warn),
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

    let pbr = dotrix::pbr::Extension::default();

    let ui = dotrix::ui::Extension::default();

    dotrix::run(studio, |extensions| {
        extensions.load(assets);
        extensions.load(pbr);
        extensions.load(ui);
    });
}
