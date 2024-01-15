//! Identifiers module provides `Id` for assets and other engine-related entities

use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

pub use uuid::Uuid;

/*
pub const MESHES_NAMESPACE: u64 = ASSETS_NAMESPACE | 0x01;
pub const IMAGES_NAMESPACE: u64 = ASSETS_NAMESPACE | 0x02;
pub const AUDIOS_NAMESPACE: u64 = ASSETS_NAMESPACE | 0x03;
pub const VIDEOS_NAMESPACE: u64 = ASSETS_NAMESPACE | 0x04;
pub const TEXTS_NAMESPACE: u64 = ASSETS_NAMESPACE | 0x05;
pub const SHADERS_NAMESPACE: u64 = ASSETS_NAMESPACE | 0x06;

pub const GPU_NAMESPACE: u64 = DOTRIX_NAMESPACE | 0x0200;
pub const BUFFERS_NAMESPACE: u64 = GPU_NAMESPACE | 0x01;
pub const TEXTURES_NAMESPACE: u64 = GPU_NAMESPACE | 0x02;
pub const SAMPLER_NAMESPACE: u64 = SAMPLER_NAMESPACE | 0x03;

pub const ENTITIES_NAMESPACE: u64 = DOTRIX_NAMESPACE | 0x0300;
*/

/// Asset identifier
pub struct Id<T> {
    /// Actual identifier value
    value: uuid::Uuid,
    phantom: PhantomData<T>,
}

impl<T> Id<T> {
    /// Constructs new random id
    pub fn new() -> Self {
        Self {
            value: uuid::Uuid::new_v4(),
            phantom: PhantomData,
        }
    }

    /// Construct new null id
    pub fn null() -> Self {
        Self::default()
    }

    /// Checks if id is null
    pub fn is_null(&self) -> bool {
        self.value.is_nil()
    }

    /// Returns reference to internal Uuid instance
    pub fn uuid(&self) -> &uuid::Uuid {
        &self.value
    }

    /// Clones the Id under another type cast
    pub fn cast<N>(&self) -> Id<N> {
        Id::from(self.uuid().clone())
    }
}

impl<T> From<uuid::Uuid> for Id<T> {
    fn from(value: uuid::Uuid) -> Self {
        Self {
            value,
            phantom: PhantomData,
        }
    }
}

impl<T> From<(u64, u64)> for Id<T> {
    fn from(value: (u64, u64)) -> Self {
        let (high, low) = value;
        Self {
            value: Uuid::from_u64_pair(high, low),
            phantom: PhantomData,
        }
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let of_type = std::any::type_name::<T>().split("::").last().unwrap();
        write!(
            f,
            "Id<{}>({:?})",
            of_type,
            self.value.hyphenated().to_string()
        )
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id {
            value: self.value,
            phantom: PhantomData,
        }
    }
}

impl<T> Copy for Id<T> {}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self {
            value: uuid::Uuid::nil(),
            phantom: PhantomData,
        }
    }
}

unsafe impl<T> Send for Id<T> {}
unsafe impl<T> Sync for Id<T> {}

#[cfg(test)]
mod tests {
    #[test]
    fn id_map_can_restore_data() {}
}
