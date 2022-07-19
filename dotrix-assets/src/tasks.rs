use crate::{Assets, Bundle, File, Resource};
use dotrix_core as dotrix;

#[derive(Default)]
pub struct LoadTask {}

impl dotrix::Task for LoadTask {
    type Context = (dotrix::Any<File>,);
    type Provides = Bundle;

    fn run(&mut self, (file,): Self::Context) -> Self::Provides {
        Bundle {
            path: file.path.clone(),
            assets: Vec::new(),
            version: 0,
            last_modified: None,
        }
    }
}

#[derive(Default)]
pub struct StoreTask {}

impl dotrix::Task for StoreTask {
    type Context = (dotrix::Any<Resource>, dotrix::Mut<Assets>);
    type Provides = Resource;

    fn run(&mut self, (resource, assets): Self::Context) -> Self::Provides {
        Resource {
            path: resource.path.clone(),
            last_modified: resource.last_modified,
            version: resource.version,
            assets: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct Watchdog {
    pub hot_reload: bool,
}

impl dotrix::Task for Watchdog {
    type Context = (dotrix::Ref<Assets>, dotrix::Ref<dotrix::Tasks>);
    type Provides = ();

    fn run(&mut self, (assets, tasks): Self::Context) -> Self::Provides {}
}
