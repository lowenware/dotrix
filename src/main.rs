// ------------------------------------------------------------------------------------------------

fn main() {
    /*
    let mut task_manager = dotrix::TaskManager::new(8);

    task_manager.store(SomeStorage::default());

    let mut rng = rand::thread_rng();

    for file in [
        "model-male.gltf",
        "model-female-sexy.gltf",
        "model-house.gltf",
        "model-loggers-camp.gltf",
        "texture-sand.png",
        "skin-troll.png",
        "intro.avi",
        "styles.css",
    ] {
        let delay = rng.gen_range(0..5);
        task_manager.add(AssetLoader::new(file, delay));
    }

    task_manager.add(AssetReader {});
    task_manager.add(SceneBuilder {});
    task_manager.add(Renderer {});
    */

    let settings = dotrix::Settings::default();
    let application = dotrix::application(settings);

    application.run();
    /*
    loop {
        println!("!!! Starting cycle------------------------------------------------------------");
        let now = std::time::Instant::now();
        task_manager.run();
        task_manager.wait();
        println!(
            "!!! Executed in {}us",
            (std::time::Instant::now() - now).as_micros()
        );
    }
    */
}
