use dotrix;
use rand;
use rand::Rng;

const EDITOR_TITLE: &str = "Dotrix 3D Editor";

struct AssetMeta {
    name: String,
}

struct AssetData {
    name: String,
    size: usize,
    index: u32,
}

#[derive(Default)]
struct Assets {
    list: Vec<(u32, String)>,
}

// TASK `AssetReader`
struct AssetReader {
    name: String,
    delay: u64,
}

impl AssetReader {
    fn new(name: &str, delay: u64) -> Self {
        Self {
            name: String::from(name),
            delay,
        }
    }
}

impl dotrix::Task for AssetReader {
    type Context = ();
    type Provides = AssetMeta;

    fn run(&mut self, ctx: Self::Context) -> Self::Provides {
        println!("--> AssetReader ({}), delay={}", self.name, self.delay);
        std::thread::sleep(std::time::Duration::from_secs(self.delay));
        AssetMeta {
            name: self.name.clone(),
        }
    }
}

// TASK `AssetLoader`
struct AssetLoader;

impl dotrix::Task for AssetLoader {
    type Context = (dotrix::Any<AssetMeta>,);
    type Provides = AssetData;

    fn run(&mut self, (meta,): Self::Context) -> Self::Provides {
        println!("--> AssetLoader ({})", meta.name);
        AssetData {
            name: meta.name.clone(),
            size: meta.name.len(),
            index: meta.index(),
        }
    }
}

// Task `AssetCollector`
struct AssetCollector;

impl dotrix::Task for AssetCollector {
    type Context = (
        dotrix::State<dotrix::Ro<()>>,
        dotrix::All<AssetData>,
        dotrix::Rw<Assets>,
    );
    type Provides = dotrix::Done;

    fn run(&mut self, (_state, data, mut assets): Self::Context) -> Self::Provides {
        println!("--> AssetCollector (Any State)");
        let assets_number = data.count();
        for asset in data.iter() {
            println!("  - {}: {}", asset.index, asset.name);
            assets.list.push((asset.index, asset.name.clone()));
        }
        println!("--> collected {} assets", assets_number);
        dotrix::Done::default()
    }
}

struct MyState {
    num: u32,
}

struct StatedTask {}

impl dotrix::Task for StatedTask {
    type Context = (dotrix::State<dotrix::Ro<MyState>>,);
    type Provides = ();

    fn run(&mut self, (state,): Self::Context) -> Self::Provides {
        println!("--> StatedTask (Must be called on MyState only)");
        state.pop();
    }
}

fn main() {
    let mut task_manager = dotrix::TaskManager::new(8);

    task_manager.store(Assets::default());

    println!("Adding tasks...");
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
        task_manager.add(AssetReader::new(file, delay));
    }

    task_manager.add(AssetLoader {});
    task_manager.add(AssetCollector {});

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
}
