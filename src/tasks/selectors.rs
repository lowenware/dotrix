use std::marker::PhantomData;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use crate::context;

/// Imutable accessor for [`context::Context`] instance
pub struct Ref<T>
where
    T: context::Context,
{
    data: *const T,
}

pub struct Mut<T>
where
    T: context::Context,
{
    data: *mut T,
}

impl<T> context::Selector for Mut<T>
where
    T: context::Context,
{
    type DataSlot = T;

    fn select(manager: &context::Manager, _: &context::Dependencies) -> Option<Self> {
        manager.global::<T>()
            .and_then(|slot| Self {
                data: slot.data.get()
            })
    }

    fn lock_type() -> Option<(std::any::TypeId, context::LockType)> {
        Some((std::any::TypeId::of::<T>(), context::LockType::Mut))
    }
}

impl<T> Deref for Mut<T>
where
    T: context::Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for Mut<T>
where
    T: context::Context,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

unsafe impl<T: context::Context> Send for Mut<T> {}
unsafe impl<T: context::Context> Sync for Mut<T> {}


/// context::Selector for provision of any dependency
#[derive(Debug)]
pub struct Any<T: context::Context> {
    data: *const T,
    index: u32,
    total: u32,
}

/// context::Selector for provision of all dependencies
#[derive(Debug)]
pub struct All<T: context::Context> {
    list: *const Vec<Box<dyn std::any::Any + Send + 'static>>,
    _phantom: PhantomData<T>,
}

/// State selector
pub struct State<T: context::Selector> {
    selector: T,
    changes: Arc<Mutex<VecDeque<context::StatesStackOperation>>>,
}

/// context::Selector that takes ownership over selected data
pub struct Take<T: context::Selector> {
    selector: T
}

/// context::Selector that does not create a dependency on selected data
pub struct Try<T: context::Selector> {
    selector: T
}