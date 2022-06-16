use core::ops::{Deref, DerefMut};
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::{scheduler, task};

pub trait Context: std::any::Any + 'static {}
impl<T: 'static> Context for T {}

#[derive(Default)]
struct Data {
    name: String,
    instances: Vec<Box<dyn std::any::Any + Send + 'static>>,
    instances_count: u32,
    providers: u32,
    notify: bool,
}

impl From<&scheduler::Task> for Data {
    fn from(task: &scheduler::Task) -> Self {
        Data {
            name: String::from(task.provides_as_str()),
            ..Default::default()
        }
    }
}

///  Context Manager
pub struct Manager {
    contexts: HashMap<TypeId, Box<dyn std::any::Any + Send + 'static>>,
    data: HashMap<TypeId, Data>,
}

// NOTE: It is safe to use in combination with Locker
unsafe impl Sync for Manager {}

impl Manager {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            data: HashMap::new(),
        }
    }

    fn get<T: Context + std::any::Any>(&self) -> Option<&T> {
        self.contexts
            .get(&TypeId::of::<T>())
            .map(|srv| srv.downcast_ref::<T>().unwrap())
    }

    fn get_mut<T: Context + std::any::Any>(&mut self) -> Option<&mut T> {
        self.contexts
            .get_mut(&TypeId::of::<T>())
            .map(|srv| srv.downcast_mut::<T>().unwrap())
    }

    fn get_data<T: Context + std::any::Any>(&self) -> Option<&Data> {
        self.data.get(&TypeId::of::<T>())
    }

    pub fn store(&mut self, type_id: TypeId, context: Box<dyn std::any::Any + Send + 'static>) {
        self.contexts.insert(type_id, context);
    }

    pub fn discard(&mut self, type_id: TypeId) {
        self.contexts.remove(&type_id);
    }

    /// Register dependecy data
    pub fn register(&mut self, type_id: TypeId) {
        let mut entry = self.data.entry(type_id).or_insert(Data::default());
        entry.providers += 1;
    }

    pub fn register_provider(&mut self, task: &scheduler::Task) {
        let mut entry = self.data.entry(task.provides()).or_insert(Data::from(task));
        entry.providers += 1;
    }

    pub fn provide(&mut self, type_id: TypeId, data: Box<dyn std::any::Any + Send + 'static>) {
        if let Some(entry) = self.data.get_mut(&type_id) {
            entry.instances.push(data);
            entry.instances_count += 1;
        }
    }

    fn provide_fake(&mut self, type_id: TypeId) {
        if let Some(entry) = self.data.get_mut(&type_id) {
            entry.instances_count += 1;
        }
    }

    pub fn subscribe(&mut self, type_id: TypeId) {
        let mut entry = self.data.entry(type_id).or_insert(Data::default());
        entry.notify = true;
    }

    pub fn unsubscribe(&mut self, type_id: TypeId) {
        if let Some(entry) = self.data.get_mut(&type_id) {
            entry.notify = false;
        }
    }

    pub fn match_dependencies(
        &self,
        dependencies: &Dependencies,
        skip_accessor_all: bool,
    ) -> Option<Dependencies> {
        // NOTE: when task has several Any<T> dependencies (keep in mind, that every task has
        // implicit Any<scheduler::Start> dependency), the condition to run is to have at least one provision
        // of each Any<T> dependency with at least one new provision
        let mut result = dependencies.clone();
        let mut has_any_condition = false;
        let mut has_all_condition = false;
        let mut meet_any_condition = false;
        let mut task_was_executed = false;
        for (type_id, dependency) in dependencies.data.iter() {
            if let Some(entry) = self.data.get(&type_id) {
                let instances_len = entry.instances_count;
                match dependency {
                    DependencyType::Any(index) => {
                        has_any_condition = true;
                        if instances_len > 0 {
                            if *index < instances_len {
                                meet_any_condition = true;
                            }
                            result
                                .data
                                .insert(*type_id, DependencyType::Any(*index + 1));
                            continue;
                        }
                    }
                    DependencyType::All(count) => {
                        has_all_condition = true;
                        if skip_accessor_all {
                            return None;
                        }
                        if *count == instances_len {
                            task_was_executed = true;
                        }
                        result
                            .data
                            .insert(*type_id, DependencyType::All(instances_len));
                        if entry.providers == instances_len {
                            continue;
                        }
                    }
                }
            }
            return None;
        }

        if !meet_any_condition && !(!has_any_condition && !task_was_executed) {
            return None;
        }

        return Some(result);
    }

    pub fn fetch<T>(&self, dependencies: &Dependencies) -> T
    where
        T: TupleAccessor,
    {
        T::fetch(self, dependencies)
    }

    pub fn reset_data(&mut self) {
        for entry in self.data.values_mut() {
            entry.instances.clear();
            entry.instances_count = 0;
            entry.providers = 0;
        }
    }

    fn cleanup_graph(&mut self) {
        for entry in self.data.values_mut() {
            if entry.instances.len() == 0 && entry.instances_count != 0 {
                entry.instances_count = 0;
            }
        }
    }

    pub fn rebuild_graph(
        &mut self,
        pool: &HashMap<task::Id, scheduler::TaskSlot>,
        queue: &[task::Id],
    ) {
        println!("Context: Rebuild graph");
        let mut skip_accessor_all = true;
        let mut dependencies: HashMap<task::Id, Dependencies> = HashMap::new();
        loop {
            let mut executed_something = false;
            // TODO: queue, not pool
            for slot in pool.values() {
                if let Some(task) = slot.task.as_ref() {
                    let task_dependencies = dependencies
                        .entry(task.id())
                        .or_insert(task.dependencies().clone());

                    if let Some(dependencies_state) =
                        self.match_dependencies(task_dependencies, skip_accessor_all)
                    {
                        self.register_provider(task);
                        // TODO: add fake provision
                        self.provide_fake(task.provides());
                        dependencies.insert(task.id(), dependencies_state);
                        executed_something = true;
                    }
                }
            }
            if executed_something {
                if !skip_accessor_all {
                    skip_accessor_all = true
                }
            } else {
                if !skip_accessor_all {
                    self.cleanup_graph();
                    break;
                } else {
                    skip_accessor_all = false;
                }
            }
        }
        for entry in self.data.values() {
            println!("  > {} has {} providers", entry.name, entry.providers);
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum LockType {
    Ro(u32),
    Rw,
}

#[derive(Debug)]
pub struct Lock {
    data: Vec<(TypeId, LockType)>,
}

pub struct LockManager {
    data: HashMap<TypeId, LockType>,
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn lock(&mut self, lock: &Lock) -> bool {
        for (type_id, lock_type) in lock.data.iter() {
            if let Some(existing_lock) = self.data.get(type_id) {
                if *lock_type == LockType::Rw || *existing_lock == LockType::Rw {
                    return false;
                }
            }
        }
        for (type_id, lock_type) in lock.data.iter() {
            if let LockType::Ro(counter) = self.data.entry(*type_id).or_insert(*lock_type) {
                *counter += 1;
            }
        }
        return true;
    }

    pub fn unlock(&mut self, lock: &Lock) {
        for (type_id, _) in lock.data.iter() {
            let mut remove = false;
            if let Some(existing_lock) = self.data.get_mut(type_id) {
                match existing_lock {
                    LockType::Ro(count) => {
                        if *count == 1 {
                            remove = true;
                        } else {
                            *count -= 1;
                        }
                    }
                    LockType::Rw => {
                        remove = true;
                    }
                };
            }
            if remove {
                self.data.remove(&type_id);
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DependencyType {
    Any(u32),
    All(u32),
}

#[derive(Debug, Default)]
pub struct Dependencies {
    data: HashMap<TypeId, DependencyType>,
}

impl Clone for Dependencies {
    fn clone(&self) -> Self {
        Self {
            data: self
                .data
                .iter()
                .map(|(key, entry)| {
                    (
                        *key,
                        match entry {
                            DependencyType::Any(_) => DependencyType::Any(0),
                            DependencyType::All(_) => DependencyType::All(0),
                        },
                    )
                })
                .collect::<HashMap<_, _>>(),
        }
    }
}

pub trait TupleAccessor: Sized {
    fn fetch(context_manager: &Manager, dependencies: &Dependencies) -> Self;
    fn lock() -> Lock;
    fn dependencies() -> Dependencies;
}

macro_rules! impl_tuple_accessor {
    (($($i: ident),*)) => {
        impl<$($i,)*> TupleAccessor for ($($i,)*)
        where
            $($i: Accessor,)*
        {
            #[allow(unused)]
            fn fetch(context_manager: &Manager, dependencies: &Dependencies) -> Self {
                (
                    $($i::fetch(context_manager, dependencies).expect("Context to be stored"),)*
                )
            }

            fn lock() -> Lock {
                Lock {
                    data: [ $($i::lock(),)* ]
                        .into_iter()
                        .filter(|l: &Option<(std::any::TypeId, LockType)>| l.is_some())
                        .map(|l| l.unwrap())
                        .collect::<Vec<_>>(),
                }
            }

            fn dependencies() -> Dependencies {
                let mut data = [ $($i::dependency(),)* ]
                    .into_iter()
                    .filter(|d: &Option<(std::any::TypeId, DependencyType)>| d.is_some())
                    .map(|d| d.unwrap())
                    .collect::<HashMap<_, _>>();
                data.insert(
                    std::any::TypeId::of::<scheduler::Start>(),
                    DependencyType::Any(0)
                );
                Dependencies {
                    data
                }
            }
        }
    }
}

impl_tuple_accessor!(());
impl_tuple_accessor!((A));
impl_tuple_accessor!((A, B));
impl_tuple_accessor!((A, B, C));
impl_tuple_accessor!((A, B, C, D));
impl_tuple_accessor!((A, B, C, D, E));
impl_tuple_accessor!((A, B, C, D, E, F));
impl_tuple_accessor!((A, B, C, D, E, F, G));
impl_tuple_accessor!((A, B, C, D, E, F, G, H));
impl_tuple_accessor!((A, B, C, D, E, F, G, H, I));
impl_tuple_accessor!((A, B, C, D, E, F, G, H, I, J));
impl_tuple_accessor!((A, B, C, D, E, F, G, H, I, J, K));
impl_tuple_accessor!((A, B, C, D, E, F, G, H, I, J, K, L));
impl_tuple_accessor!((A, B, C, D, E, F, G, H, I, J, K, L, M));
impl_tuple_accessor!((A, B, C, D, E, F, G, H, I, J, K, L, M, N));
impl_tuple_accessor!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O));
impl_tuple_accessor!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P));

/// Abstraction to access Service in the storage
pub trait Accessor: Sized + Send + Sync {
    /// Type of Service to be accessed
    type Data: Context;
    /// Fetches the Service from the storage
    fn fetch(context: &Manager, dependencies: &Dependencies) -> Option<Self>;
    /// Returns Data type and lock type
    fn lock() -> Option<(std::any::TypeId, LockType)>;
    /// Returns Dependency
    fn dependency() -> Option<(std::any::TypeId, DependencyType)>;
}

impl<T> Accessor for Rw<T>
where
    T: Context,
{
    type Data = T;

    fn fetch(manager: &Manager, _: &Dependencies) -> Option<Self> {
        manager.get::<T>().map(|data_ref| Self {
            data: (data_ref as *const T) as *mut T,
        })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Rw))
    }

    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        None
    }
}

/// Mutable accessor for [`Context`] instance
pub struct Rw<T>
where
    T: Context,
{
    data: *mut T,
}

impl<T> Deref for Rw<T>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for Rw<T>
where
    T: Context,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

unsafe impl<T: Context> Send for Rw<T> {}
unsafe impl<T: Context> Sync for Rw<T> {}

/// Imutable accessor for [`Context`] instance
pub struct Ro<T>
where
    T: Context,
{
    data: *const T,
}

impl<T> Deref for Ro<T>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

unsafe impl<T: Context> Send for Ro<T> {}
unsafe impl<T: Context> Sync for Ro<T> {}

impl<T> Accessor for Ro<T>
where
    T: Context,
{
    type Data = T;

    fn fetch(manager: &Manager, _: &Dependencies) -> Option<Self> {
        manager.get::<T>().map(|data_ref| Self {
            data: data_ref as *const T,
        })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Ro(0)))
    }

    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        None
    }
}

/// Accessor for provision of any dependency
#[derive(Debug)]
pub struct Any<T: Context> {
    data: *const T,
    index: u32,
    total: u32,
}

impl<T: Context> Any<T> {
    pub fn index(&self) -> u32 {
        self.index
    }
    pub fn total(&self) -> u32 {
        self.total
    }
}

impl<T> Deref for Any<T>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

unsafe impl<T: Context> Send for Any<T> {}
unsafe impl<T: Context> Sync for Any<T> {}

impl<T> Accessor for Any<T>
where
    T: Context,
{
    type Data = T;

    fn fetch(manager: &Manager, dependencies: &Dependencies) -> Option<Self> {
        let index = match dependencies
            .data
            .get(&std::any::TypeId::of::<T>())
            .expect("Dependency to be consistant")
        {
            DependencyType::Any(index) => *index,
            _ => panic!("Dependency and accessor missmatch"),
        };
        manager
            .get_data::<T>()
            .map(|d| {
                if d.instances.len() > index as usize {
                    return Some(Self {
                        data: d.instances[index as usize].downcast_ref::<T>().unwrap(),
                        total: d.providers,
                        index,
                    });
                }
                None
            })
            .unwrap_or(None)
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        None
    }

    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        Some((std::any::TypeId::of::<T>(), DependencyType::Any(0)))
    }
}

/// Accessor for provision of all dependencies
#[derive(Debug)]
pub struct All<T: Context> {
    list: *const Vec<Box<dyn std::any::Any + Send + 'static>>,
    index: usize,
    _phantom: PhantomData<T>,
}

pub struct AllIter<'a, T> {
    iter: std::slice::Iter<'a, Box<dyn std::any::Any + Send + 'static>>,
    _phantom: PhantomData<T>,
}

impl<T: Context> All<T> {
    pub fn count(&self) -> usize {
        unsafe { &*self.list }.len()
    }

    pub fn iter<'a>(&'a self) -> AllIter<'a, T> {
        AllIter {
            iter: unsafe { &*self.list }.iter(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> Iterator for AllIter<'a, T>
where
    T: Context,
{
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|item| item.downcast_ref::<T>().unwrap())
    }
}

unsafe impl<T: Context> Send for All<T> {}
unsafe impl<T: Context> Sync for All<T> {}

impl<T> Accessor for All<T>
where
    T: Context,
{
    type Data = T;

    fn fetch(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        manager.get_data::<T>().map(|d| All {
            list: &d.instances,
            index: 0,
            _phantom: PhantomData,
        })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        None
    }

    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        Some((std::any::TypeId::of::<T>(), DependencyType::All(0)))
    }
}
