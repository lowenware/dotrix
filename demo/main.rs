// mod camera;
// mod states;
// mod ui;
mod scene;

use dotrix::log;

pub struct Demo {
    application_name: String,
    version: u32,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            application_name: String::from("Dotrix Demo"),
            version: env!("CARGO_PKG_VERSION_MAJOR")
                .parse::<u32>()
                .ok()
                .unwrap_or(0),
        }
    }
}

impl dotrix::Application for Demo {
    fn app_version(&self) -> u32 {
        self.version
    }

    fn app_name<'a>(&'a self) -> &'a str {
        &self.application_name
    }

    fn startup(
        self,
        scheduler: &dotrix::tasks::Scheduler,
        display: &mut dotrix::graphics::Display,
    ) {
        log::info!("Startup");
        // Setup rendering semaphores
        let gpu = display.gpu();
        let present_complete_semaphore = gpu.create_semaphore();
        let renderer = dotrix::RenderModels::setup()
            .surface_format(display.surface_format())
            .wait_semaphores([present_complete_semaphore.clone()])
            .create(gpu.clone());
        let render_complete_semaphore = renderer.complete_semaphore().clone();

        display.set_render_complete_semaphore(render_complete_semaphore);
        display.set_present_complete_semaphore(present_complete_semaphore);

        // add Assets context
        scheduler.add_context(dotrix::Assets::default());
        // add World context
        scheduler.add_context(dotrix::World::default());
        // add spawner tasks
        scheduler.add_task(scene::SpawnEntities::default());
        // add rendering task
        scheduler.add_task(renderer);
    }
}

fn main() {
    // Initialize logging
    dotrix::Log::default()
        .level("dotrix", log::LevelFilter::Debug)
        .level("*", log::LevelFilter::Debug)
        .subscribe();

    // Run application
    dotrix::run(Demo::default());
}
