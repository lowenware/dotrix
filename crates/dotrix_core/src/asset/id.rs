use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

pub type RawId = u64;

pub struct Id<T> {
    pub id: RawId,
    phantom: PhantomData<T>
}

impl<T> Id<T> {
    pub fn new(id: RawId) -> Self {
        Self {
            id,
            phantom: PhantomData,
        }
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
        write!(f, "<{}:{}>", of_type, self.id)
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
