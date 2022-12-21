use crate::{Assets, Bundle, File, LoadError, Resource};
use dotrix_core as dotrix;
use dotrix_log as log;
use std::collections::HashMap;

use dotrix::{Any, Take};

#[derive(Default)]
pub struct LoadTask {}

impl dotrix::Task for LoadTask {
    type Context = (dotrix::Any<File>, dotrix::Ref<Assets>);
    type Output = Bundle;

    fn run(&mut self, (file, assets): Self::Context) -> Self::Output {
        use std::io::Read;

        /*
        let file_name = file
            .path
            .file_stem()
            .map(|n| n.to_str().unwrap())
            .unwrap()
            .to_string();

        let extension = file
            .path
            .extension()
            .map(|e| e.to_str().unwrap())
            .unwrap_or("")
            .to_string();
        */

        let mut list = None;

        for loader in assets.loaders() {
            if loader.can_load(&file.path) {
                if let Ok(mut fs_file) = std::fs::File::open(&file.path) {
                    let mut data = Vec::with_capacity(file.size);
                    if fs_file.read_to_end(&mut data).is_ok() {
                        list = Some(loader.load(&file.path, data));
                    } else {
                        log::error!("Could not read file {:?}", &file.path);
                    }
                } else {
                    log::error!("Could not open file {:?}", &file.path);
                }
                break;
            }
        }

        if list.is_none() {
            println!("No loader found for {:?}", &file.path);
        }

        Bundle {
            path: file.path.clone(),
            assets: list.unwrap_or_else(Vec::new),
        }
    }
}

#[derive(Default)]
pub struct StoreTask {}

impl dotrix::Task for StoreTask {
    type Context = (Take<Any<Bundle>>, dotrix::Mut<Assets>);
    type Output = Resource;

    fn run(&mut self, (mut bundle, mut assets): Self::Context) -> Self::Output {
        use crate::Asset;

        let mut meta = Vec::<(String, String)>::with_capacity(bundle.assets.len());
        while let Some(asset) = bundle.assets.pop() {
            let name = asset.name().to_string();
            let type_name = asset.type_name().to_string();
            assets.store_raw(asset);
            meta.push((type_name, name));
        }

        let mut resource = assets.resource(bundle.path.clone());
        // NOTE: in theory we can remove old assets here

        resource.version += 1;
        resource.assets = meta;
        resource.clone()
    }
}

pub struct Watchdog {
    pub hot_reload: bool,
    pub registry: HashMap<std::path::PathBuf, std::time::SystemTime>,
}

impl Default for Watchdog {
    fn default() -> Self {
        Self {
            hot_reload: false,
            registry: HashMap::new(),
        }
    }
}

/*
impl dotrix::Task for Watchdog {
    type Context = (dotrix::Ref<Assets>, dotrix::Ref<dotrix::Tasks>);
    type Output = ();

    fn run(&mut self, (assets, tasks): Self::Context) -> Self::Output {
        for resource in assets.resources() {
            if let Ok(metadata) = std::fs::metadata(&resource.path) {
                if let Ok(resource_modified) = metadata.modified() {
                    if let Some(last_modified) = self.registry.get(&resource.path) {
                        if resource_modified == *last_modified {
                            continue;
                        }
                    }

                    self.registry
                        .insert(resource.path.clone(), resource_modified);

                    tasks.provide(File {
                        path: resource.path.clone(),
                        size: metadata.len() as usize,
                    });
                }
            }
        }
    }
}
*/
