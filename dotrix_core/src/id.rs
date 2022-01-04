//! Assets identifiers
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

/// Asset identifier
pub struct Id<T> {
    /// Actual identifier value
    pub id: u64,
    phantom: PhantomData<T>,
}

impl<T> Id<T> {
    /// Constructs an asset identifier from u64
    pub fn new(id: u64) -> Self {
        Self {
            id,
            phantom: PhantomData,
        }
    }

    /// Checks if id is null
    pub fn is_null(&self) -> bool {
        self.id == 0
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let of_type = std::any::type_name::<T>().split("::").last().unwrap();
        write!(f, "[Id<{}>: {}]", of_type, self.id)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id {
            id: self.id,
            phantom: PhantomData,
        }
    }
}

impl<T> Copy for Id<T> {}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new(0)
    }
}
