//! Identifiers module provides `Id` for assets and other engine-related entities

use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

pub use uuid::Uuid;

/*
pub const IMAGES_NAMESPACE: u64 = ASSETS_NAMESPACE | 0x01;
pub const MESHES_NAMESPACE: u64 = ASSETS_NAMESPACE | 0x02;
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

/// Id namespace abstraction
pub trait NameSpace {
    /// Returns low 8 bytes
    fn namespace() -> u64
    where
        Self: Sized;

    /// Returns ID from the namespace with defined high bytes
    fn id(high: u64) -> Id<Self>
    where
        Self: Sized,
    {
        Id::new(Self::namespace(), high)
    }
}

impl<T> Id<T> {
    /// Construct new Id from parts
    pub fn new(low: u64, high: u64) -> Self {
        Self {
            value: uuid::Uuid::from_u64_pair(low, high),
            phantom: PhantomData,
        }
    }

    /// Constructs new random id
    pub fn random() -> Self {
        Self {
            value: uuid::Uuid::new_v4(),
            phantom: PhantomData,
        }
    }

    /// Checks if id is null
    pub fn is_null(&self) -> bool {
        self.value.is_nil()
    }

    /// Returns reference to internal Uuid instance
    pub fn uuid(&self) -> &uuid::Uuid {
        &self.value
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

#[cfg(test)]
mod tests {
    #[test]
    fn id_map_can_restore_data() {}
}
