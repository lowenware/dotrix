use crate::{scheduler, task};
use dotrix_log as log;
use dotrix_types::Id;
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

/// Memory slot to keep global data
pub struct GlobalSlot {
    pub data: UnsafeCell<Box<dyn std::any::Any + Send + 'static>>,
}

/// Memory slot to keep data provided as a task output
#[derive(Default)]
pub struct OutputSlot {
    name: String,
    instances: Vec<UnsafeCell<Option<Box<dyn std::any::Any + Send + 'static>>>>,
    providers: usize,
    /// Protected cells keep data if Some() on reset
    protected: bool,
}

pub struct StateSlot {
    pub id: std::any::TypeId,
    pub name: String,
    pub data: UnsafeCell<Box<dyn std::any::Any + Send + 'static>>,
}

///  Context Manager
pub struct Manager {
    globals: HashMap<TypeId, GlobalSlot>,
    outputs: HashMap<TypeId, OutputSlot>,
    states_stack: Vec<StateSlot>,
    states_changes: Arc<Mutex<VecDeque<StatesStackOperation>>>,
}

impl GlobalSlot {
    fn new<T: Context>(context: T) -> Self
    where
        T: std::any::Any + Send + 'static,
    {
        Self {
            data: UnsafeCell::new(Box::new(context)),
        }
    }
}

impl From<Box<dyn std::any::Any + Send + 'static>> for GlobalSlot {
    fn from(boxed: Box<dyn std::any::Any + Send + 'static>) -> Self {
        Self {
            data: UnsafeCell::new(boxed),
        }
    }
}

impl From<&scheduler::Task> for OutputSlot {
    fn from(task: &scheduler::Task) -> Self {
        Self {
            name: String::from(task.output_as_str()),
            instances: vec![],
            providers: 0,
            protected: false,
        }
    }
}

// NOTE: It is safe to use in combination with Locker
unsafe impl Sync for Manager {}

impl Manager {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            outputs: HashMap::new(),
            states_stack: vec![StateSlot::from(())],
            states_changes: Arc::new(Mutex::new(VecDeque::with_capacity(4))),
        }
    }

    unsafe fn select_mut<T: Context>(&self) -> Option<Mut<T>> {
        self.globals
            .get(&std::any::TypeId::of::<T>())
            .and_then(|slot| (&mut *slot.data.get()).downcast_mut::<T>())
            .map(|data| Mut {
                data: data as *mut T,
            })
    }

    unsafe fn select_ref<T: Context>(&self) -> Option<Ref<T>> {
        self.globals
            .get(&std::any::TypeId::of::<T>())
            .and_then(|slot| (&*slot.data.get()).downcast_ref::<T>())
            .map(|data| Ref {
                data: data as *const T,
            })
    }

    unsafe fn select_any<T: Context>(&self, index: usize) -> Option<Any<T>> {
        self.outputs
            .get(&std::any::TypeId::of::<T>())
            .and_then(|slot| {
                let total = slot.instances.len();
                (&*slot.instances[index].get())
                    .as_ref()
                    .and_then(|data| data.downcast_ref::<T>())
                    .map(|data| (data, total))
            })
            .map(|(data, total)| Any {
                data: data as *const T,
                index,
                total,
            })
    }

    unsafe fn select_all<T: Context>(&self) -> Option<All<T>> {
        self.outputs.get(&std::any::TypeId::of::<T>()).map(|slot| {
            let data = slot
                .instances
                .iter()
                .filter(|data| (&*data.get()).is_some())
                .map(|data| {
                    (&*data.get())
                        .as_ref()
                        .unwrap()
                        .downcast_ref::<T>()
                        .unwrap() as *const T
                })
                .collect::<Vec<_>>();

            All { data }
        })
    }

    unsafe fn take_any<T: Context>(&self, index: usize) -> Option<Take<Any<T>>> {
        self.outputs
            .get(&std::any::TypeId::of::<T>())
            .and_then(|slot| {
                let total = slot.instances.len();
                (&mut *slot.instances[index].get())
                    .take()
                    .and_then(|data| data.downcast::<T>().ok())
                    .map(|data| (data, total))
            })
            .map(|(data, total)| Take {
                selection: Any {
                    data: Box::<T>::into_raw(data) as *const T,
                    index,
                    total,
                },
            })
    }

    unsafe fn take_all<T: Context>(&self) -> Option<Take<All<T>>> {
        self.outputs.get(&std::any::TypeId::of::<T>()).map(|slot| {
            let data = slot
                .instances
                .iter()
                .map(|data| (&mut *data.get()).take())
                .filter(|data| data.is_some())
                .map(|data| Box::<T>::into_raw(data.unwrap().downcast::<T>().unwrap()) as *const T)
                .collect::<Vec<_>>();
            Take {
                selection: All { data },
            }
        })
    }

    unsafe fn state_ref<T: Context + std::any::Any>(&self) -> Option<State<Ref<T>>> {
        let state_id = TypeId::of::<T>();
        let state = if state_id == TypeId::of::<()>() {
            self.states_stack.first()
        } else {
            self.states_stack.last()
        };

        state
            .and_then(|state| (&*state.data.get()).downcast_ref::<T>())
            .map(|state| State {
                selection: Ref {
                    data: state as *const T,
                },
                changes: Arc::clone(&self.states_changes),
            })
    }

    unsafe fn state_mut<T: Context + std::any::Any>(&self) -> Option<State<Mut<T>>> {
        let state_id = TypeId::of::<T>();
        let state = if state_id == TypeId::of::<()>() {
            self.states_stack.first()
        } else {
            self.states_stack.last()
        };

        state
            .and_then(|state| (&mut *state.data.get()).downcast_mut::<T>())
            .map(|state| State {
                selection: Mut {
                    data: state as *mut T,
                },
                changes: Arc::clone(&self.states_changes),
            })
    }

    pub fn store_as<T: std::any::Any + Send + 'static>(&mut self, context: T) {
        self.globals
            .insert(std::any::TypeId::of::<T>(), GlobalSlot::new(context));
    }

    pub fn store_boxed(
        &mut self,
        type_id: TypeId,
        context: Box<dyn std::any::Any + Send + 'static>,
    ) {
        self.globals.insert(type_id, GlobalSlot::from(context));
    }

    pub fn discard(&mut self, type_id: TypeId) {
        self.globals.remove(&type_id);
    }

    /// Register dependecy data
    pub fn register(&mut self, type_id: TypeId, name: String, providers: usize, protected: bool) {
        let entry = self.outputs.entry(type_id).or_insert(OutputSlot {
            name,
            protected,
            providers,
            ..Default::default()
        });
    }

    pub fn register_provider(&mut self, task: &scheduler::Task) {
        let entry = self
            .outputs
            .entry(task.output_type_id())
            .or_insert(OutputSlot::from(task));
        entry.providers += 1;
    }

    pub fn provide(&mut self, type_id: TypeId, data: Box<dyn std::any::Any + Send + 'static>) {
        let entry = self.outputs.entry(type_id).or_insert(OutputSlot::default());
        entry.instances.push(UnsafeCell::new(Some(data)));
        log::debug!(
            "Provide {} -> {} of {}",
            entry.name,
            entry.instances.len(),
            entry.providers
        );
    }

    pub fn output_name(&self, type_id: &TypeId) -> Option<&str> {
        self.outputs.get(type_id).map(|slot| slot.name.as_str())
    }

    fn provide_fake(&mut self, type_id: TypeId) {
        if let Some(entry) = self.outputs.get_mut(&type_id) {
            entry.instances.push(UnsafeCell::new(None));
        }
    }

    pub fn current_state(&self) -> TypeId {
        let state = self
            .states_stack
            .last()
            .expect("There always must be a state");

        log::debug!("Current state: {} ({:?})", state.name, state.id);
        state.id
    }

    pub fn match_dependencies(&self, dependencies: &Dependencies) -> Option<Dependencies> {
        let mut result = dependencies.clone();
        for (type_id, dependency) in dependencies.data.iter() {
            let entry = match self.outputs.get(&type_id) {
                Some(dependency) => dependency,
                None => {
                    return None;
                }
            };
            let instances_len = entry.instances.len();
            match dependency {
                DependencyType::Any(index) => {
                    if instances_len > 0 && *index < instances_len {
                        result
                            .data
                            .insert(*type_id, DependencyType::Any(*index + 1));
                        continue;
                    } else {
                        return None;
                    }
                }
                DependencyType::All(count) => {
                    if *count == 0 && instances_len >= entry.providers {
                        result
                            .data
                            .insert(*type_id, DependencyType::All(instances_len));
                        continue;
                    } else {
                        return None;
                    }
                }
            }
        }

        Some(result)
    }

    pub unsafe fn fetch<T>(&self, dependencies: &Dependencies) -> T
    where
        T: ContextSelector,
    {
        T::select_context(self, dependencies)
    }

    pub fn reset_data(&mut self, reset_providers: bool) {
        let loop_type_id = std::any::TypeId::of::<scheduler::Loop>();
        for (type_id, entry) in self.outputs.iter_mut() {
            if entry.protected {
                unsafe {
                    entry.instances.retain(|data| (&*data.get()).is_some());
                }
            } else {
                entry.instances.clear();
                if reset_providers {
                    entry.providers = 0;
                }
            }
        }
    }

    unsafe fn cleanup_graph(&mut self) {
        for entry in self.outputs.values_mut() {
            // can't use clear here, because some control data should stay in the pool
            entry.instances.retain(|data| (&*data.get()).is_some());
            // if entry.instances.len() == 0 && entry.instances.len()_count != 0 {
            //     entry.instances_count = 0;
            //}
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

    pub unsafe fn calculate_providers<T: Context>(
        &mut self,
        pool: &task::Pool,
        queue: &[Id<task::Slot>],
    ) {
        let loop_providers = pool.calculate_context_providers(queue, TypeId::of::<T>(), self);

        if loop_providers != 1 {
            log::warn!("Invalid Loop providers number: {}", loop_providers);
        }
    }

    pub fn set_output_providers(&mut self, type_id: TypeId, providers: usize) {
        if let Some(slot) = self.outputs.get_mut(&type_id) {
            if !slot.protected {
                slot.providers = providers;
            }
        } else {
            log::warn!("No slot for type {:?}. Missing task?", type_id);
        }
    }

    /*
    pub unsafe fn rebuild_graph(
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
    */
}

pub enum StatesStackOperation {
    Push(StateSlot),
    Pop,
    PopUntil(std::any::TypeId),
}

pub trait Context: std::any::Any + 'static {}
impl<T: 'static> Context for T {}

impl StateSlot {
    fn from<T: std::any::Any + Send + 'static>(data: T) -> Self {
        Self {
            id: std::any::TypeId::of::<T>(),
            name: String::from(std::any::type_name::<T>()),
            data: UnsafeCell::new(Box::new(data)),
        }
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
    Any(usize),
    All(usize),
}

impl DependencyType {
    pub fn reset(&mut self) {
        match self {
            DependencyType::Any(index) => *index = 0,
            DependencyType::All(count) => *count = 0,
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
pub struct Dependencies {
    pub data: HashMap<TypeId, DependencyType>,
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
                            DependencyType::Any(_) => DependencyType::Any(0),
                            DependencyType::All(_) => DependencyType::All(0),
                        },
                    )
                })
                .collect::<HashMap<_, _>>(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SelectorTarget {
    Global,
    Output(DependencyType),
    State,
}

/// Selector of specific context data from the context manager
pub trait Selector: Sized + Send + Sync {
    /// Type of Service to be accessed
    type DataSlot: Context;

    /// Returns State Dependency
    fn target() -> (std::any::TypeId, SelectorTarget);

    /// Fetches the Service from the storage
    unsafe fn select(context: &Manager, dependencies: &Dependencies) -> Option<Self>;

    /// Returns DataSlot type and lock type
    fn lock_type() -> Option<(std::any::TypeId, LockType)> {
        None
    }

    /// Very unsafe, only selectors like Take<T> can call this
    unsafe fn drop_data(&mut self) {}

    // / Returns Dependency
    //fn dependency_type() -> Option<(std::any::TypeId, DependencyType)> {
    //    None
    //}
}

/// Selector of a complete context tuple from the context manager
pub trait ContextSelector: Sized {
    unsafe fn select_context(manager: &Manager, dependencies: &Dependencies) -> Self;
    fn lock() -> Lock;
    fn dependencies() -> Dependencies;
    fn states() -> Vec<std::any::TypeId>;
}

macro_rules! impl_context_selector {
    (($($i: ident),*)) => {
        impl<$($i,)*> ContextSelector for ($($i,)*)
        where
            $($i: Selector,)*
        {
            #[allow(unused)]
            unsafe fn select_context(manager: &Manager, dependencies: &Dependencies) -> Self {
                (
                    $($i::select(manager, dependencies).unwrap_or_else(
                        || panic!("Failed to fetch ({})", std::any::type_name::<$i>())
                    ),)*
                )
            }

            fn lock() -> Lock {
                Lock {
                    data: [ $($i::lock_type(),)* ]
                        .into_iter()
                        .filter(|l: &Option<(std::any::TypeId, LockType)>| l.is_some())
                        .map(|l| l.unwrap())
                        .collect::<Vec<_>>(),
                }
            }

            fn states() -> Vec<std::any::TypeId> {
                let mut has_default_state = false;
                [
                    (
                        std::any::TypeId::of::<scheduler::Loop>(),
                        SelectorTarget::Output(DependencyType::Any(0))
                    ),
                    $($i::target(),)*
                ]
                    .into_iter()
                    .filter_map(|(type_id, target)| match target {
                        SelectorTarget::State => {
                            if type_id == TypeId::of::<()>() {
                                has_default_state = true;
                            }
                            Some(type_id)
                        },
                        _ => None
                    })
                    .collect::<Vec<_>>()

                // if !has_default_state {
                //     states.push(TypeId::of::<()>());
                // }
                // states
            }

            fn dependencies() -> Dependencies {
                let data = [
                    (
                        std::any::TypeId::of::<scheduler::Loop>(),
                        SelectorTarget::Output(DependencyType::Any(0))
                    ),
                    $($i::target(),)*
                ]
                    .into_iter()
                    .filter_map(|(type_id, target)| match target {
                        SelectorTarget::Output(dependency_type) => Some(
                            (type_id, dependency_type)
                        ),
                        _ => None
                    })
                    .collect::<HashMap<_, _>>();

                Dependencies {
                    data
                }
            }
        }
    }
}

impl_context_selector!(());
impl_context_selector!((A));
impl_context_selector!((A, B));
impl_context_selector!((A, B, C));
impl_context_selector!((A, B, C, D));
impl_context_selector!((A, B, C, D, E));
impl_context_selector!((A, B, C, D, E, F));
impl_context_selector!((A, B, C, D, E, F, G));
impl_context_selector!((A, B, C, D, E, F, G, H));
impl_context_selector!((A, B, C, D, E, F, G, H, I));
impl_context_selector!((A, B, C, D, E, F, G, H, I, J));
impl_context_selector!((A, B, C, D, E, F, G, H, I, J, K));
impl_context_selector!((A, B, C, D, E, F, G, H, I, J, K, L));
impl_context_selector!((A, B, C, D, E, F, G, H, I, J, K, L, M));
impl_context_selector!((A, B, C, D, E, F, G, H, I, J, K, L, M, N));
impl_context_selector!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O));
impl_context_selector!((A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P));

pub struct Ref<T>
where
    T: Context,
{
    data: *const T,
}

pub struct Mut<T>
where
    T: Context,
{
    data: *mut T,
}

/// Selector for provision of any dependency
#[derive(Debug)]
pub struct Any<T: Context> {
    data: *const T,
    index: usize,
    total: usize,
}

/// Selector for provision of all dependencies
#[derive(Debug)]
pub struct All<T: Context> {
    data: Vec<*const T>,
}

/// context::Selector that takes ownership over selected data
pub struct Take<T: Selector> {
    selection: T,
}

pub struct State<T: Selector> {
    selection: T,
    changes: Arc<Mutex<VecDeque<StatesStackOperation>>>,
}

impl<T> Selector for Mut<T>
where
    T: Context,
{
    type DataSlot = T;

    fn target() -> (std::any::TypeId, SelectorTarget) {
        (std::any::TypeId::of::<T>(), SelectorTarget::Global)
    }

    unsafe fn select(manager: &Manager, _: &Dependencies) -> Option<Self> {
        manager.select_mut::<T>()
    }

    fn lock_type() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Mut))
    }
}

impl<T> Selector for Ref<T>
where
    T: Context,
{
    type DataSlot = T;

    fn target() -> (std::any::TypeId, SelectorTarget) {
        (std::any::TypeId::of::<T>(), SelectorTarget::Global)
    }

    unsafe fn select(manager: &Manager, _: &Dependencies) -> Option<Self> {
        manager.select_ref::<T>()
    }

    fn lock_type() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Ref(0)))
    }
}

impl<T> Selector for Any<T>
where
    T: Context,
{
    type DataSlot = T;

    fn target() -> (std::any::TypeId, SelectorTarget) {
        (
            std::any::TypeId::of::<T>(),
            SelectorTarget::Output(DependencyType::Any(0)),
        )
    }

    unsafe fn select(manager: &Manager, dependencies: &Dependencies) -> Option<Self> {
        let index = match dependencies
            .data
            .get(&std::any::TypeId::of::<T>())
            .expect("Dependency to be consistant")
        {
            DependencyType::Any(index) => *index as usize,
            _ => panic!("Dependency and accessor missmatch"),
        };
        manager.select_any(index)
    }

    fn lock_type() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Ref(0)))
    }

    unsafe fn drop_data(&mut self) {
        if !self.data.is_null() {
            let _ = unsafe { *Box::from_raw(self.data as *mut T) };
        }
    }
}

impl<T> Selector for All<T>
where
    T: Context,
{
    type DataSlot = T;

    fn target() -> (std::any::TypeId, SelectorTarget) {
        (
            std::any::TypeId::of::<T>(),
            SelectorTarget::Output(DependencyType::All(0)),
        )
    }

    unsafe fn select(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        manager.select_all::<T>()
    }

    fn lock_type() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Ref(0)))
    }

    unsafe fn drop_data(&mut self) {
        while let Some(ptr) = self.data.pop() {
            let _ = unsafe { *Box::from_raw(ptr as *mut T) };
        }
    }
}

impl<T> Selector for Take<Any<T>>
where
    T: Context,
{
    type DataSlot = T;

    fn target() -> (std::any::TypeId, SelectorTarget) {
        (
            std::any::TypeId::of::<T>(),
            SelectorTarget::Output(DependencyType::Any(0)),
        )
    }

    unsafe fn select(manager: &Manager, dependencies: &Dependencies) -> Option<Self> {
        let index = match dependencies
            .data
            .get(&std::any::TypeId::of::<T>())
            .expect("Dependency to be consistant")
        {
            DependencyType::Any(index) => *index as usize,
            _ => panic!("Dependency and accessor missmatch"),
        };

        manager.take_any::<T>(index)
    }

    fn lock_type() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Mut))
    }
}

impl<T> Selector for Take<All<T>>
where
    T: Context,
{
    type DataSlot = T;

    fn target() -> (std::any::TypeId, SelectorTarget) {
        (
            std::any::TypeId::of::<T>(),
            SelectorTarget::Output(DependencyType::All(0)),
        )
    }

    unsafe fn select(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        manager.take_all::<T>()
    }

    fn lock_type() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Mut))
    }
}

impl<T> Selector for State<Ref<T>>
where
    T: Context,
{
    type DataSlot = T;

    fn target() -> (std::any::TypeId, SelectorTarget) {
        (std::any::TypeId::of::<T>(), SelectorTarget::State)
    }

    unsafe fn select(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        manager.state_ref::<T>()
    }

    fn lock_type() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Ref(0)))
    }
}

impl<T> Selector for State<Mut<T>>
where
    T: Context,
{
    type DataSlot = T;

    fn target() -> (std::any::TypeId, SelectorTarget) {
        (std::any::TypeId::of::<T>(), SelectorTarget::State)
    }

    unsafe fn select(manager: &Manager, _dependencies: &Dependencies) -> Option<Self> {
        manager.state_mut::<T>()
    }

    fn lock_type() -> Option<(std::any::TypeId, LockType)> {
        Some((std::any::TypeId::of::<T>(), LockType::Mut))
    }
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

impl<T> Deref for Ref<T>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
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

impl<T> Deref for Take<Any<T>>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.selection.data }
    }
}

impl<T> DerefMut for Take<Any<T>>
where
    T: Context,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.selection.data as *mut T) }
    }
}

impl<T> Deref for State<Ref<T>>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.selection.data }
    }
}

impl<T> Deref for State<Mut<T>>
where
    T: Context,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.selection.data }
    }
}

impl<T> DerefMut for State<Mut<T>>
where
    T: Context,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.selection.data }
    }
}

impl<T: Context> Any<T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn total(&self) -> usize {
        self.total
    }
}

impl<T: Context> All<T> {
    pub fn iter<'a>(&'a self) -> AllIter<'a, T> {
        AllIter {
            inner: self.data.iter(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct AllIter<'i, T> {
    inner: std::slice::Iter<'i, *const T>,
}

impl<'i, T: Context> Iterator for AllIter<'i, T> {
    type Item = &'i T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|i| unsafe { &**i })
    }
}

impl<T: Context> Take<Any<T>> {
    pub fn take(mut self) -> T {
        unsafe {
            let result = *Box::from_raw(self.selection.data as *mut T);
            self.selection.data = std::ptr::null();
            result
        }
    }
}

impl<T: Context> Take<All<T>> {
    pub fn take(mut self) -> Vec<T> {
        unsafe {
            let result = self
                .selection
                .data
                .iter()
                .map(|i| *Box::from_raw(*i as *mut T))
                .collect::<Vec<_>>();
            self.selection.data.clear();
            result
        }
    }

    pub fn drain<'a>(&'a mut self) -> TakeAllIter<'a, T> {
        let len = self.selection.data.len();
        TakeAllIter {
            inner: self.selection.data.drain(0..len),
        }
    }
}

pub struct TakeAllIter<'a, T> {
    inner: std::vec::Drain<'a, *const T>,
}

impl<'a, T: Context> Iterator for TakeAllIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|i| unsafe { *Box::from_raw(i as *mut T) })
    }
}

impl<T: Selector> Drop for Take<T> {
    fn drop(&mut self) {
        unsafe {
            self.selection.drop_data();
        }
    }
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

unsafe impl<T: Context> Send for Ref<T> {}
unsafe impl<T: Context> Sync for Ref<T> {}
unsafe impl<T: Context> Send for Mut<T> {}
unsafe impl<T: Context> Sync for Mut<T> {}
unsafe impl<T: Context> Send for Any<T> {}
unsafe impl<T: Context> Sync for Any<T> {}
unsafe impl<T: Context> Send for All<T> {}
unsafe impl<T: Context> Sync for All<T> {}
