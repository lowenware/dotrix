use dotrix;
use rand;
use rand::Rng;

/// Raw asset data loaded from disc
struct RawAsset {
    name: String,
}

/// Ready to use asset
struct Asset {
    name: String,
    size: usize,
    index: u32,
}

/// Scene data
struct Scene {
    entities_count: usize,
}

/// Shared storage for assets
#[derive(Default)]
struct SomeStorage {
    list: Vec<(u32, String)>,
}

/// TASK `AssetLoader` ----------------------------------------------------------------------------
struct AssetLoader {
    name: String,
    delay: u64,
}

impl AssetLoader {
    fn new(name: &str, delay: u64) -> Self {
        Self {
            name: String::from(name),
            delay,
        }
    }
}

impl dotrix::Task for AssetLoader {
    type Context = ();
    type Provides = RawAsset;

    fn run(&mut self, ctx: Self::Context) -> Self::Provides {
        println!("--> AssetLoader ({}), delay={}", self.name, self.delay);
        std::thread::sleep(std::time::Duration::from_secs(self.delay));
        RawAsset {
            name: self.name.clone(),
        }
    }
}

/// TASK `AssetReader` ----------------------------------------------------------------------------
struct AssetReader;

impl dotrix::Task for AssetReader {
    type Context = (dotrix::Any<RawAsset>,);
    type Provides = Asset;

    fn run(&mut self, (meta,): Self::Context) -> Self::Provides {
        println!("--> AssetReader ({})", meta.name);
        Asset {
            name: meta.name.clone(),
            size: meta.name.len(),
            index: meta.index(),
        }
    }
}

/// Task `SceneBuilder` ---------------------------------------------------------------------------
struct SceneBuilder;

impl dotrix::Task for SceneBuilder {
    type Context = (
        dotrix::State<dotrix::Ref<()>>,
        dotrix::All<Asset>,
        dotrix::Mut<SomeStorage>,
    );
    type Provides = Scene;

    fn run(&mut self, (_state, data, mut assets): Self::Context) -> Self::Provides {
        println!("--> SceneBuilder (Any State)");
        let assets_number = data.count();
        for asset in data.iter() {
            println!("  - {}: {}", asset.index, asset.name);
            assets.list.push((asset.index, asset.name.clone()));
        }
        Scene {
            entities_count: assets_number,
        }
    }
}

/// Task `Renderer` -------------------------------------------------------------------------------
struct Renderer;

impl dotrix::Task for Renderer {
    type Context = (dotrix::Any<Scene>,);
    type Provides = ();
    // type Provides = dotrix::Done;

    fn run(&mut self, (scene,): Self::Context) -> Self::Provides {
        println!("--> Renderer ({} entities)", scene.entities_count);
        // dotrix::Done::default()
    }
}

struct MyState {
    num: u32,
}

/// Stated Task -----------------------------------------------------------------------------------
struct StatedTask {}

impl dotrix::Task for StatedTask {
    type Context = (dotrix::State<dotrix::Ref<MyState>>,);
    type Provides = ();

    fn run(&mut self, (state,): Self::Context) -> Self::Provides {
        println!("--> StatedTask (Must be called on MyState only)");
        state.pop();
    }
}

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
