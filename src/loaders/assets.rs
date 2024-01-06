use std::collections::HashMap;

use uuid::Uuid;

use crate::utils::Id;

/// Asset control abstraction trait
pub trait Asset: Send + 'static {
    /// Returns [`std::any::TypeId`] of the asset type
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    /// Returns name of the asset type
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Returns name of the asset
    fn name(&self) -> &str;
}

impl dyn Asset {
    /// Returns true if the asset is of type T
    #[inline]
    pub fn is<T: Asset>(&self) -> bool {
        let t = std::any::TypeId::of::<T>();
        let concrete = self.type_id();
        t == concrete
    }

    /// Checks asset type and downcasts dynamic reference
    #[inline]
    pub fn downcast_ref<T: Asset>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn Asset as *const T)) }
        } else {
            None
        }
    }

    /// Checks asset type and downcasts dynamic mutable reference
    #[inline]
    pub fn downcast_mut<T: Asset>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn Asset as *mut T)) }
        } else {
            None
        }
    }
}

/// Assets library
#[derive(Default)]
pub struct Assets {
    /// Index of IDs assigned by asset name
    registry: HashMap<String, uuid::Uuid>,
    /// Id indexed assets map
    map: HashMap<uuid::Uuid, Box<dyn Asset>>,
}

impl Assets {
    /// Constructs new [`Assets`] instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores an asset and returns [`Id`] of it
    pub fn set<T: Asset>(&mut self, asset: T) -> Id<T> {
        let uuid_u64_pair = self.store(Box::new(asset));
        Id::from(uuid_u64_pair)
    }

    /// Searches an asset by its [`Id`] and returns it by a reference if the asset exists
    pub fn get<T: Asset>(&self, id: Id<T>) -> Option<&T> {
        self.map
            .get(id.uuid())
            .and_then(|asset| asset.downcast_ref::<T>())
    }

    /// Searches an asset by its [`Id`] and returns it by a mutual reference if the asset exists
    pub fn get_mut<T: Asset>(&mut self, id: Id<T>) -> Option<&mut T> {
        self.map
            .get_mut(id.uuid())
            .and_then(|asset| asset.downcast_mut::<T>())
    }

    /// Removes an asset from the Service and returns it if the asset exists
    pub fn remove<T: Asset>(&mut self, id: Id<T>) -> Option<T> {
        self.map.remove(id.uuid()).map(|asset| {
            *(unsafe { Box::from_raw((Box::leak(asset) as *mut dyn Asset) as *mut T) })
        })
    }

    /// Stores an already boxed asset
    pub fn store(&mut self, asset: Box<dyn Asset>) -> (u64, u64) {
        let asset_name = asset.name();
        let uuid = self
            .registry
            .entry(String::from(asset.name()))
            .or_insert_with(|| Uuid::new_v4())
            .clone();

        self.map.insert(uuid, asset);
        uuid.as_u64_pair()
    }

    /// Searches for an asset by the name and return [`Id`] of it if the asset exists
    pub fn find<T: Asset>(&self, name: &str) -> Option<Id<T>> {
        self.registry.get(name).map(|uuid| Id::from(*uuid))
    }
}
