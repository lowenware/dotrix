use dotrix::log::{error, info, warn};

struct Studio {}

impl Studio {
    fn new() -> Self {
        Self {}
    }
}

impl dotrix::Application for Studio {
    fn configure(&self, settings: &mut dotrix::Settings) {
        settings.title = String::from("Dotrix Studio");
    }

    fn init(&self, manager: &dotrix::Manager) {}
}

fn main() {
    dotrix::log::subscribe(dotrix::Log::default());

    let studio = Studio::new();

    let assets = dotrix::assets::Extension {
        root: std::path::PathBuf::from("./resources"),
        init: |assets| {
            assets.install(dotrix::image::Loader::default());
            assets.install(dotrix::shader::Loader::default());
        },
        hot_reload: true,
    };

    let camera = dotrix::camera::Extension::default();

    dotrix::run(studio, |core| {
        core.extend_with(assets);
        core.extend_with(camera);
    });
}
