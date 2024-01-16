// mod camera;
// mod states;
// mod ui;
mod scene;

use dotrix::log;

const APP_NAME: &str = "Dotrix Demo";

fn main() {
    // Initialize logging
    dotrix::Log::default()
        .level("dotrix", log::LevelFilter::Debug)
        .level("*", log::LevelFilter::Debug)
        .subscribe();

    // get version from Cargo.toml
    let version = env!("CARGO_PKG_VERSION_MAJOR")
        .parse::<u32>()
        .ok()
        .unwrap_or(0);

    // Setup dotrix core
    let (mut display, gpu, event_loop) = dotrix::Core::setup()
        .application_name(APP_NAME)
        .application_version(version)
        .workers(8)
        .fps_request(Some(30.0))
        .create()
        .into_tuple();

    // Setup rendering semaphores
    let present_complete_semaphore = gpu.create_semaphore();
    let renderer = dotrix::RenderModels::setup()
        .surface_format(display.surface_format())
        .wait_semaphores([present_complete_semaphore.clone()])
        .create(gpu.clone());
    let render_complete_semaphore = renderer.complete_semaphore().clone();

    display.set_render_complete_semaphore(render_complete_semaphore);
    display.set_present_complete_semaphore(present_complete_semaphore);

    // run application
    dotrix::run(event_loop, |scheduler| {
        // add Assets context
        scheduler.add_context(dotrix::Assets::default());
        // add Display context
        scheduler.add_context(display);
        // add World context
        scheduler.add_context(dotrix::World::default());
        // add spawner tasks
        scheduler.add_task(scene::SpawnEntities::default());
        // add rendering task
        scheduler.add_task(renderer);
    });
}
