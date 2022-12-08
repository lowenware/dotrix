use core::ops::{Deref, DerefMut};
use std::any::TypeId;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::{scheduler, task};

///  Context Manager
pub struct Manager {
    // TODO: introduce ServiceSlot
    services: HashMap<TypeId, Box<dyn std::any::Any + Send + 'static>>,
    data: HashMap<TypeId, DataSlot>,
    states_stack: Vec<StateSlot>,
    states_changes: Arc<Mutex<VecDeque<StatesStackOperation>>>,
}

// NOTE: It is safe to use in combination with Locker
unsafe impl Sync for Manager {}

impl Manager {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            data: HashMap::new(),
            states_stack: vec![StateSlot::from(())],
            states_changes: Arc::new(Mutex::new(VecDeque::with_capacity(4))),
        }
    }

    fn get<T: Context + std::any::Any>(&self) -> Option<&T> {
        self.services
            .get(&TypeId::of::<T>())
            .map(|srv| srv.downcast_ref::<T>().unwrap())
    }

    fn state<T: Context + std::any::Any>(&self) -> Option<&T> {
        let state_id = TypeId::of::<T>();
        let state = if state_id == TypeId::of::<()>() {
            self.states_stack.first()
        } else {
            self.states_stack.last()
        };
        state.map(|state| state.data.downcast_ref::<T>().expect("Valid states stack"))
    }

    fn get_mut<T: Context + std::any::Any>(&mut self) -> Option<&mut T> {
        self.services
            .get_mut(&TypeId::of::<T>())
            .map(|srv| srv.downcast_mut::<T>().unwrap())
    }

    fn get_data<T: Context + std::any::Any>(&self) -> Option<&DataSlot> {
        self.data.get(&TypeId::of::<T>())
    }

    pub fn store_as<T: std::any::Any + Send + 'static>(&mut self, context: T) {
        self.services
            .insert(std::any::TypeId::of::<T>(), Box::new(context));
    }

    pub fn store(&mut self, type_id: TypeId, context: Box<dyn std::any::Any + Send + 'static>) {
        self.services.insert(type_id, context);
    }

    pub fn discard(&mut self, type_id: TypeId) {
        self.services.remove(&type_id);
    }

    /// Register dependecy data
    pub fn register(&mut self, type_id: TypeId) {
        let mut entry = self.data.entry(type_id).or_insert(DataSlot::default());
        entry.providers += 1;
    }

    pub fn register_provider(&mut self, task: &scheduler::Task) {
        let mut entry = self
            .data
            .entry(task.output_type_id())
            .or_insert(DataSlot::from(task));
        entry.providers += 1;
    }

    pub fn provide(&mut self, type_id: TypeId, data: Box<dyn std::any::Any + Send + 'static>) {
        let mut entry = self.data.entry(type_id).or_insert(DataSlot::default());
        entry.instances.push(data);
        entry.instances_count += 1;
    }

    fn provide_fake(&mut self, type_id: TypeId) {
        if let Some(entry) = self.data.get_mut(&type_id) {
            entry.instances_count += 1;
        }
    }

    pub fn match_states(&self, states: &[TypeId]) -> bool {
        let current_state_id = self.states_stack.last().unwrap().id;
        for state_id in states.iter() {
            if *state_id != current_state_id {
                return false;
            }
        }
        true
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
        // let mut has_all_condition = false;
        let mut meet_any_condition = false;
        let mut task_was_executed = false;
        for (type_id, dependency) in dependencies.data.iter() {
            if let Some(entry) = self.data.get(&type_id) {
                let instances_len = entry.instances_count;
                match dependency {
                    DependencyType::None => {
                        if skip_accessor_all {
                            return None;
                        }
                        continue;
                    }
                    DependencyType::Take => {
                        if instances_len > 0 {
                            continue;
                        }
                    }
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
                        // has_all_condition = true;
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
        T: TupleSelector,
    {
        T::fetch(self, dependencies)
    }

    pub fn reset_data(&mut self, reset_providers: bool) {
        let loop_type_id = std::any::TypeId::of::<scheduler::Loop>();
        for (type_id, entry) in self.data.iter_mut() {
            if entry.providers != 0 || *type_id == loop_type_id {
                entry.instances.clear();
                entry.instances_count = 0;
                if reset_providers {
                    entry.providers = 0;
                }
            }
        }
    }

    fn cleanup_graph(&mut self) {
        for entry in self.data.values_mut() {
            if entry.instances.len() == 0 && entry.instances_count != 0 {
                entry.instances_count = 0;
            }
        }
    }

    pub fn apply_states_changes(&mut self) {
        let mut changes = self.states_changes.lock().expect("Mutex to be locked");
        while let Some(operation) = changes.pop_front() {
            match operation {
                StatesStackOperation::Push(state) => {
                    self.states_stack.push(state);
                }
                StatesStackOperation::Pop => {
                    if self.states_stack.len() > 1 {
                        self.states_stack.pop();
                    }
                }
                StatesStackOperation::PopUntil(state_id) => {
                    while self.states_stack.len() > 1
                        && self.states_stack.last().unwrap().id != state_id
                    {
                        self.states_stack.pop();
                    }
                }
            }
        }
    }

    pub fn rebuild_graph(
        &mut self,
        pool: &HashMap<task::Id, scheduler::TaskSlot>,
        queue: &[task::Id],
    ) {
        // println!("Context: Rebuild graph");
        let mut skip_accessor_all = true;
        let mut dependencies: HashMap<task::Id, Dependencies> = HashMap::new();
        loop {
            let mut executed_something = false;
            for task_id in queue.iter() {
                if let Some(slot) = pool.get(task_id) {
                    if let Some(task) = slot.task.as_ref() {
                        let task_dependencies = dependencies
                            .entry(task.id())
                            .or_insert(task.dependencies().clone());

                        if let Some(dependencies_state) =
                            self.match_dependencies(task_dependencies, skip_accessor_all)
                        {
                            self.register_provider(task);
                            // TODO: add fake provision
                            self.provide_fake(task.output_type_id());
                            dependencies.insert(task.id(), dependencies_state);
                            executed_something = true;
                        }
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
    }
}

enum StatesStackOperation {
    Push(StateSlot),
    Pop,
    PopUntil(std::any::TypeId),
}

pub trait Context: std::any::Any + 'static {}
impl<T: 'static> Context for T {}

#[derive(Default)]
struct DataSlot {
    name: String,
    instances: Vec<Box<dyn std::any::Any + Send + 'static>>,
    instances_count: u32,
    providers: u32,
}

impl From<&scheduler::Task> for DataSlot {
    fn from(task: &scheduler::Task) -> Self {
        DataSlot {
            name: String::from(task.output_as_str()),
            ..Default::default()
        }
    }
}

struct StateSlot {
    id: std::any::TypeId,
    name: String,
    data: Box<dyn std::any::Any + Send + 'static>,
}

impl StateSlot {
    fn from<T: std::any::Any + Send + 'static>(data: T) -> Self {
        Self {
            id: std::any::TypeId::of::<T>(),
            name: String::from(std::any::type_name::<T>()),
            data: Box::new(data),
        }
    }
}

pub struct State<T: Selector> {
    selector: T,
    changes: Arc<Mutex<VecDeque<StatesStackOperation>>>,
}

impl<T: Selector> State<T> {
    pub fn push<D: std::any::Any + Send + 'static>(&self, data: D) {
        self.changes
            .lock()
            .expect("Mutex to be locked")
            .push_back(StatesStackOperation::Push(StateSlot::from(data)));
    }

    pub fn pop(&self) {
        self.changes
            .lock()
            .expect("Mutex to be locked")
            .push_back(StatesStackOperation::Pop);
    }

    pub fn pop_until<D: std::any::Any + Send + 'static>(&self) {
        self.changes
            .lock()
            .expect("Mutex to be locked")
            .push_back(StatesStackOperation::PopUntil(std::any::TypeId::of::<D>()));
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum LockType {
    Ref(u32),
    Mut,
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
                if *lock_type == LockType::Mut || *existing_lock == LockType::Mut {
                    return false;
                }
            }
        }
        for (type_id, lock_type) in lock.data.iter() {
            if let LockType::Ref(counter) = self.data.entry(*type_id).or_insert(*lock_type) {
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
                    LockType::Ref(count) => {
                        if *count == 1 {
                            remove = true;
                        } else {
                            *count -= 1;
                        }
                    }
                    LockType::Mut => {
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
    None,
    Any(u32),
    All(u32),
    Take,
}

impl DependencyType {
    pub fn reset(&mut self) {
        match self {
            DependencyType::Any(count) => *count = 0,
            DependencyType::All(count) => *count = 0,
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
pub struct Dependencies {
    data: HashMap<TypeId, DependencyType>,
}

impl Dependencies {
    pub fn reset(&mut self) {
        for entry in self.data.values_mut() {
            entry.reset();
        }
    }
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
                            DependencyType::None => DependencyType::None,
                            DependencyType::Any(_) => DependencyType::Any(0),
                            DependencyType::All(_) => DependencyType::All(0),
                            DependencyType::Take => DependencyType::Take,
                        },
                    )
                })
                .collect::<HashMap<_, _>>(),
        }
    }
}

pub trait TupleSelector: Sized {
    fn fetch(context_manager: &Manager, dependencies: &Dependencies) -> Self;
    fn lock() -> Lock;
    fn dependencies() -> Dependencies;
    fn states() -> Vec<std::any::TypeId>;
}

macro_rules! impl_tuple_accessor {
    (($($i: ident),*)) => {
        impl<$($i,)*> TupleSelector for ($($i,)*)
        where
            $($i: Selector,)*
        {
            #[allow(unused)]
            fn fetch(context_manager: &Manager, dependencies: &Dependencies) -> Self {
                (
                    $($i::fetch(context_manager, dependencies).unwrap_or_else(
                        || panic!("Failed to fetch '{}'", std::any::type_name::<$i>())
                    ),)*
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

            fn states() -> Vec<std::any::TypeId> {
                [ $($i::state(),)* ]
                    .into_iter()
                    .filter(|l: &Option<std::any::TypeId>| l.is_some())
                    .map(|l| l.unwrap())
                    .collect::<Vec<_>>()
            }

            fn dependencies() -> Dependencies {
                let mut data = [ $($i::dependency(),)* ]
                    .into_iter()
                    .filter(|d: &Option<(std::any::TypeId, DependencyType)>| d.is_some())
                    .map(|d| d.unwrap())
                    .collect::<HashMap<_, _>>();
                data.insert(
                    std::any::TypeId::of::<scheduler::Loop>(),
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
pub trait Selector: Sized + Send + Sync {
    /// Type of Service to be accessed
    type DataSlot: Context;
    /// Fetches the Service from the storage
    fn fetch(context: &Manager, dependencies: &Dependencies) -> Option<Self>;
    /// Returns DataSlot type and lock type
    fn lock() -> Option<(std::any::TypeId, LockType)> {
        None
    }
    /// Returns Dependency
    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        None
    }
    /// Returns State Dependency
    fn state() -> Option<std::any::TypeId> {
        None
    }
}

impl<T> Selector for Mut<T>
where
    T: Context,
{
    type DataSlot = T;

    fn fetch(manager: &Manager, _: &Dependencies) -> Option<Self> {
        manager.get::<T>().map(|data_ref| Self {
            data: (data_ref as *const T) as *mut T,
        })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Mut))
    }
}

/// Mutable accessor for [`Context`] instance
pub struct Mut<T>
where
    T: Context,
{
    data: *mut T,
}

impl<T> Deref for Mut<T>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for Mut<T>
where
    T: Context,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

unsafe impl<T: Context> Send for Mut<T> {}
unsafe impl<T: Context> Sync for Mut<T> {}

/// Imutable accessor for [`Context`] instance
pub struct Ref<T>
where
    T: Context,
{
    data: *const T,
}

impl<T> Deref for Ref<T>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

unsafe impl<T: Context> Send for Ref<T> {}
unsafe impl<T: Context> Sync for Ref<T> {}

impl<T> Selector for Ref<T>
where
    T: Context,
{
    type DataSlot = T;

    fn fetch(manager: &Manager, _: &Dependencies) -> Option<Self> {
        manager.get::<T>().map(|data_ref| Self {
            data: data_ref as *const T,
        })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Ref(0)))
    }
}

/// Selector for provision of any dependency
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

impl<T> Selector for Any<T>
where
    T: Context,
{
    type DataSlot = T;

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
        Some((std::any::TypeId::of::<T>(), LockType::Ref(0)))
    }

    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        Some((std::any::TypeId::of::<T>(), DependencyType::Any(0)))
    }
}

/// Selector that takes ownership over dependency
#[derive(Debug)]
pub struct Take<T: Context> {
    data: T,
}

impl<T: 'static> Take<T> {
    pub fn unwrap(self) -> T {
        self.data
    }
}

impl<T> Deref for Take<T>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &self.data }
    }
}

impl<T> DerefMut for Take<T>
where
    T: Context,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut self.data }
    }
}

unsafe impl<T: Context> Send for Take<T> {}
unsafe impl<T: Context> Sync for Take<T> {}

impl<T> Selector for Take<T>
where
    T: Context,
{
    type DataSlot = T;

    fn fetch(manager: &Manager, _: &Dependencies) -> Option<Self> {
        manager
            .get_data::<T>()
            .map(|d| {
                let slot = unsafe { &mut *((d as *const DataSlot) as *mut DataSlot) };
                if slot.instances_count == 0 {
                    return None;
                }
                slot.instances_count -= 1;
                slot.instances.pop().map(|data| Self {
                    data: *(unsafe {
                        Box::from_raw((Box::leak(data) as *mut dyn std::any::Any) as *mut T)
                    }),
                })
            })
            .unwrap_or(None)
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Mut))
    }

    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        Some((std::any::TypeId::of::<T>(), DependencyType::Take))
    }
}

/// Selector for provision of all dependencies
#[derive(Debug)]
pub struct All<T: Context> {
    list: *const Vec<Box<dyn std::any::Any + Send + 'static>>,
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

impl<T> Selector for All<T>
where
    T: Context,
{
    type DataSlot = T;

    fn fetch(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        manager.get_data::<T>().map(|d| All {
            list: &d.instances,
            _phantom: PhantomData,
        })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Ref(0)))
    }

    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        Some((std::any::TypeId::of::<T>(), DependencyType::All(0)))
    }
}

/// Selector for collection of all dependencies
#[derive(Debug)]
pub struct Collect<T: Context> {
    inner: Vec<T>,
}

impl<T: Context> Collect<T> {
    pub fn count(&self) -> usize {
        self.inner.len()
    }

    pub fn collect(self) -> Vec<T> {
        self.inner
    }
}

unsafe impl<T: Context> Send for Collect<T> {}
unsafe impl<T: Context> Sync for Collect<T> {}

impl<T> Selector for Collect<T>
where
    T: Context,
{
    type DataSlot = T;

    fn fetch(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        manager.get_data::<T>().map(|d| {
            let capacity = d.instances.len();
            let list = unsafe {
                &mut *(&d.instances as *const Vec<Box<dyn std::any::Any + Send>>
                    as *mut Vec<Box<dyn std::any::Any>>)
            };
            let mut collected = Vec::with_capacity(capacity);

            for i in (0..capacity).rev() {
                collected.push(*list.pop().unwrap().downcast::<T>().unwrap());
            }

            Collect { inner: collected }
        })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Mut))
    }

    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        Some((std::any::TypeId::of::<T>(), DependencyType::All(0)))
    }
}

/// Selector for collection of all dependencies even if empty
#[derive(Debug)]
pub struct Fetch<T: Context> {
    inner: Vec<T>,
}

impl<T: Context> Fetch<T> {
    pub fn count(&self) -> usize {
        self.inner.len()
    }

    pub fn fetch(self) -> Vec<T> {
        self.inner
    }
}

unsafe impl<T: Context> Send for Fetch<T> {}
unsafe impl<T: Context> Sync for Fetch<T> {}

impl<T> Selector for Fetch<T>
where
    T: Context,
{
    type DataSlot = T;

    fn fetch(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        let empty_instances = vec![];
        manager
            .get_data::<T>()
            .map(|d| &d.instances)
            .or(Some(&empty_instances))
            .map(|instances| {
                let capacity = instances.len();
                let list = unsafe {
                    &mut *(instances as *const Vec<Box<dyn std::any::Any + Send>>
                        as *mut Vec<Box<dyn std::any::Any>>)
                };
                let mut collected = Vec::with_capacity(capacity);

                for i in (0..capacity).rev() {
                    collected.push(*list.pop().unwrap().downcast::<T>().unwrap());
                }

                Fetch { inner: collected }
            })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Mut))
    }

    fn dependency() -> Option<(std::any::TypeId, DependencyType)> {
        Some((std::any::TypeId::of::<T>(), DependencyType::None))
    }
}

impl<T> Selector for State<Ref<T>>
where
    T: Context,
{
    type DataSlot = T;

    fn fetch(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        manager.state::<T>().map(|data_ref| State {
            selector: Ref {
                data: data_ref as *const T,
            },
            changes: Arc::clone(&manager.states_changes),
        })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Ref(0)))
    }

    fn state() -> Option<std::any::TypeId> {
        Some(std::any::TypeId::of::<T>())
    }
}
//unsafe impl<T: Context> Send for State<Ref<T>> {}
//unsafe impl<T: Context> Sync for State<Ref<T>> {}

impl<T> Selector for State<Mut<T>>
where
    T: Context,
{
    type DataSlot = T;

    fn fetch(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        manager.state::<T>().map(|data_ref| State {
            selector: Mut {
                data: (data_ref as *const T) as *mut T,
            },
            changes: Arc::clone(&manager.states_changes),
        })
    }

    fn lock() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Mut))
    }

    fn state() -> Option<std::any::TypeId> {
        Some(std::any::TypeId::of::<T>())
    }
}
//unsafe impl<T: Context> Send for State<Mut<T>> {}
//unsafe impl<T: Context> Sync for State<Mut<T>> {}

impl<T> Deref for State<Ref<T>>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.selector.data }
    }
}

impl<T> Deref for State<Mut<T>>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.selector.data }
    }
}

impl<T> DerefMut for State<Mut<T>>
where
    T: Context,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.selector.data }
    }
}
