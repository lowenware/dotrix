//! Identifiers module provides `Id` for assets and other engine-related entities

use std::{
    any::TypeId,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

/// Enumeration of types of actual Id value
#[derive(Debug, Hash, Copy, Clone, Eq, PartialEq)]
pub enum ValueOf {
    /// Based on TypeId
    TypeId(TypeId),
    /// Based on u64 number
    Number(u64),
}

/// Asset identifier
pub struct Id<T> {
    /// Actual identifier value
    pub id: ValueOf,
    phantom: PhantomData<T>,
}

impl<T> Id<T> {
    /// Constructs an asset identifier from u64
    pub fn new(id: u64) -> Self {
        Self {
            id: ValueOf::Number(id),
            phantom: PhantomData,
        }
    }

    /// Checks if id is null
    pub fn is_null(&self) -> bool {
        self.id == ValueOf::Number(0)
    }

    /// Returns u64 value if it can be unwrapped
    pub fn as_u64(&self) -> Option<u64> {
        match self.id {
            ValueOf::Number(number) => Some(number),
            _ => None,
        }
    }

    /// Returns TypeId value if it can be unwrapped
    pub fn as_type_id(&self) -> Option<TypeId> {
        match self.id {
            ValueOf::TypeId(type_id) => Some(type_id),
            _ => None,
        }
    }

    /// Returns u64 value if it can be unwrapped, panics if it can't
    pub fn unwrap_u64(&self) -> u64 {
        match self.id {
            ValueOf::Number(number) => number,
            _ => panic!("Id can not be unwrapped as u64"),
        }
    }

    /// Returns TypeId value if it can be unwrapped, panics if it can't
    pub fn unwrap_type_id(&self) -> TypeId {
        match self.id {
            ValueOf::TypeId(type_id) => type_id,
            _ => panic!("Id can not be unwrapped as TypeId"),
        }
    }
}

impl<T> From<TypeId> for Id<T> {
    fn from(value: TypeId) -> Self {
        Self {
            id: ValueOf::TypeId(value),
            phantom: PhantomData,
        }
    }
}

impl<T> From<u64> for Id<T> {
    fn from(value: u64) -> Self {
        Self {
            id: ValueOf::Number(value),
            phantom: PhantomData,
        }
    }
}

/// Abstraction for type-based IP getters
pub trait OfType {
    /// Returns ID based on type of T
    ///
    /// see implementation for `Id<State>` for details
    fn of<T: std::any::Any>() -> Self;
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
        write!(f, "[Id<{}>: {:?}]", of_type, self.id)
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
