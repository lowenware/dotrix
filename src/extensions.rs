use std::any::TypeId;
use std::collections::HashMap;

use dotrix_core::{Extension, Manager};

/// Extensions loader
///
/// Adds an extension to the `Manager` and `Extensions` registry
pub struct Loader<'m, 'e> {
    manager: &'m Manager,
    extensions: &'e mut Extensions,
}

impl<'m, 'e> Loader<'m, 'e> {
    pub fn new(manager: &'m Manager, extensions: &'e mut Extensions) -> Self {
        Self {
            manager,
            extensions,
        }
    }

    pub fn load<T: Extension>(&mut self, extension: T) -> &mut Self {
        extension.load(self.manager);
        self.extensions.add(extension);
        self
    }
}

/// Extensions Registry
#[derive(Default)]
pub struct Extensions {
    registry: HashMap<TypeId, Box<dyn Extension>>,
}

impl Extensions {
    pub fn add<T: Extension>(&mut self, extension: T) {
        self.registry.insert(TypeId::of::<T>(), Box::new(extension));
    }
}
