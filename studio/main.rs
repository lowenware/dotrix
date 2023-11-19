// mod camera;
// mod states;
// mod ui;

use dotrix::log;

const APP_NAME: &str = "Dotrix Studio";

fn main() {
    // Initialize logging
    dotrix::Log::default()
        .level("dotrix", log::LevelFilter::Debug)
        .level("*", log::LevelFilter::Debug)
        .subscribe();

    let version = env!("CARGO_PKG_VERSION_MAJOR")
        .parse::<u32>()
        .ok()
        .unwrap_or(0);

    let (mut display, gpu, event_loop) = dotrix::Core::setup()
        .application_name(APP_NAME)
        .application_version(version)
        .workers(8)
        .fps_request(Some(30.0))
        .create()
        .into_tuple();

    let present_complete_semaphore = gpu.create_semaphore();
    let renderer = dotrix::Renderer::setup()
        .surface_format(display.surface_format())
        .wait_semaphores([present_complete_semaphore.clone()])
        .create(gpu.clone());
    let render_complete_semaphore = renderer.complete_semaphore().clone();

    display.set_render_complete_semaphore(render_complete_semaphore);
    display.set_present_complete_semaphore(present_complete_semaphore);

    dotrix::run(event_loop, |scheduler| {
        scheduler.add_context(display);
        // scheduler.add_context(my_context);
        scheduler.add_task(renderer);
    });
}
