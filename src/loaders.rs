//! Loaders for assets from resource files
mod assets;

mod gltf_loader;
pub use gltf_loader::GltfLoader;

pub mod image_loader;
pub use image_loader::ImageLoader;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub use crate::tasks::{Any, Mut, Take, Task};
pub use assets::{Asset, Assets};

/// Resource import request
///
/// Resource file may contain one or more assets
pub struct ResourceFile {
    /// Path to resource file
    path: std::path::PathBuf,
    /// Resource loader constructor
    loader: Box<dyn ResourceLoader>,
    /// Targets to be loaded
    targets: HashSet<ResourceTarget>,
}

/// Target asset inside of a resource to be loaded
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ResourceTarget {
    type_id: std::any::TypeId,
    name: String,
}

impl ResourceFile {
    /// Constructs new ResourceRequest
    pub fn new<T: ResourceLoader>(path: impl Into<PathBuf>, loader: T) -> Self {
        Self {
            path: path.into(),
            loader: Box::new(loader),
            targets: HashSet::new(),
        }
    }

    /// Adds a target to the ResourceRequest
    pub fn target<T: Asset>(mut self, name: impl Into<String>) -> Self {
        let target = ResourceTarget {
            type_id: std::any::TypeId::of::<T>(),
            name: name.into(),
        };
        self.targets.insert(target);
        self
    }

    /// Returns resource loader
    pub fn read(&self) -> ResourceBundle {
        self.loader.read(self.path.as_path(), &self.targets)
    }
}

pub trait ResourceLoader: 'static {
    fn read(&self, path: &Path, targets: &HashSet<ResourceTarget>) -> ResourceBundle;
}

pub struct ResourceBundle {
    pub resource: PathBuf,
    pub bundle: HashMap<ResourceTarget, Option<Box<dyn Asset>>>,
}

pub struct ResourceReport {
    pub resource: PathBuf,
    pub report: HashMap<ResourceTarget, Option<(u64, u64)>>,
}

pub struct ImportResource {}

impl Task for ImportResource {
    type Context = (Take<Any<ResourceFile>>,);
    type Output = ResourceBundle;

    fn run(&mut self, (resource,): Self::Context) -> Self::Output {
        let ResourceFile {
            path,
            loader,
            targets,
        } = resource.take();

        loader.read(&path, &targets)
    }
}

pub struct StoreAssets {}

impl Task for StoreAssets {
    type Context = (Take<Any<ResourceBundle>>, Mut<Assets>);
    type Output = ResourceReport;

    fn run(&mut self, (bundle, mut assets): Self::Context) -> Self::Output {
        let ResourceBundle { resource, bundle } = bundle.take();

        let report = bundle
            .into_iter()
            .map(|(target, asset)| (target, asset.map(|asset| assets.store(asset))))
            .collect::<HashMap<_, _>>();

        ResourceReport { resource, report }
    }
}
