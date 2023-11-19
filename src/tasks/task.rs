use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::log;
use crate::tasks::context;
use crate::utils::{Id, Lock};

/// Task output channel, defines recipient of the output
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputChannel {
    /// Send result to pool
    Pool,
    /// Send result to scheduler
    Scheduler,
}

/// Task abstraction
pub trait Task: 'static + Send + Sync + Sized {
    /// Type of task's context
    type Context: context::ContextSelector;
    /// Type of task's output
    type Output: 'static + Send;

    /// Executes the task
    fn run(&mut self, ctx: Self::Context) -> Self::Output;

    /// Returns output channel of the task
    fn output_channel(&self) -> OutputChannel {
        OutputChannel::Pool
    }

    /// Boxifies the task to be stored in pool
    fn boxify(mut self, id: Id<Slot>) -> Box<dyn Executable> {
        use context::ContextSelector;
        let task_box: TaskBox<_> = TaskBox {
            id,
            type_id: TypeId::of::<Self>(),
            output_type_id: TypeId::of::<Self::Output>(),
            output_type_name: String::from(type_name::<Self::Output>()),
            name: type_name::<Self>(),
            lock: <Self::Context>::lock(),
            dependencies: <Self::Context>::dependencies(),
            states: <Self::Context>::states(),
            dependencies_state: None,
            output_channel: self.output_channel(),
            run: move |context_manager, dependencies| unsafe {
                if let Ok(manager) = context_manager.lock() {
                    let task_context = manager.fetch::<Self::Context>(dependencies);

                    let task_result = self.run(task_context);
                    Box::new(task_result)
                } else {
                    panic!(
                        "Task {} has failed to access its context",
                        type_name::<Self>()
                    );
                }
            },
        };
        Box::new(task_box)
    }
}

/// Boxified task
pub struct TaskBox<F>
where
    F: FnMut(
        &Arc<Mutex<context::Manager>>,
        &context::Dependencies,
    ) -> Box<dyn Any + 'static + Send>,
{
    id: Id<Slot>,
    type_id: TypeId,
    output_type_id: TypeId,
    output_type_name: String,
    name: &'static str,
    lock: Vec<Lock>,
    dependencies: context::Dependencies,
    states: Vec<TypeId>,
    output_channel: OutputChannel,
    run: F,
    dependencies_state: Option<context::Dependencies>,
}

/// Abstraction for tasks independently of function signature
pub trait Executable: Send + Sync {
    /// Execute task
    fn run(
        &mut self,
        context_manager: &Arc<Mutex<context::Manager>>,
    ) -> Box<dyn Any + 'static + Send>;

    /// Get task name
    fn name(&self) -> &str;

    /// Get task id
    fn id(&self) -> Id<Slot>;

    /// Get task type id
    fn type_id(&self) -> TypeId;

    /// Get type id of result
    fn output_type_id(&self) -> TypeId;

    /// Get type id of result
    fn output_as_str(&self) -> &str;

    /// Get lock for context
    fn lock(&self) -> &[Lock];

    /// State dependencies
    fn states(&self) -> &[TypeId];

    /// Task dependencies
    fn dependencies(&self) -> &context::Dependencies;

    /// Set dependencies state for the scheduler
    fn schedule_with(&mut self, dependencies_state: context::Dependencies);

    /// Returns true if dependencies state is set
    fn is_scheduled(&self) -> bool;

    /// Reset dependencies
    fn reset(&mut self);

    /// Returns channel where output of the task must be provided
    fn output_channel(&self) -> OutputChannel;
}

impl<F> Executable for TaskBox<F>
where
    F: FnMut(
            &Arc<Mutex<context::Manager>>,
            &context::Dependencies,
        ) -> Box<dyn Any + 'static + Send>
        + Send
        + Sync,
{
    fn run(
        &mut self,
        context_manager: &Arc<Mutex<context::Manager>>,
    ) -> Box<dyn Any + 'static + Send> {
        let result = (self.run)(context_manager, &self.dependencies);
        let dependencies_state = self.dependencies_state.take().unwrap();
        self.dependencies = dependencies_state;
        result
    }

    fn id(&self) -> Id<Slot> {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn type_id(&self) -> TypeId {
        self.type_id
    }

    fn output_type_id(&self) -> TypeId {
        self.output_type_id
    }

    fn output_as_str(&self) -> &str {
        &self.output_type_name
    }

    fn output_channel(&self) -> OutputChannel {
        self.output_channel
    }

    fn lock(&self) -> &[Lock] {
        self.lock.as_slice()
    }

    fn states(&self) -> &[TypeId] {
        &self.states
    }

    fn dependencies(&self) -> &context::Dependencies {
        &self.dependencies
    }

    fn schedule_with(&mut self, dependencies_state: context::Dependencies) {
        self.dependencies_state = Some(dependencies_state);
    }

    /// Returns true if dependencies state is set
    fn is_scheduled(&self) -> bool {
        self.dependencies_state.is_some()
    }

    fn reset(&mut self) {
        self.dependencies.reset();
    }
}

/// Output type storage
pub struct Output<T> {
    _phantom: PhantomData<T>,
}

impl<T> Output<T> {
    /// Constructs new instance
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Default for Output<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory slot for the stored task
#[derive(Default)]
pub struct Slot {
    task: Option<Box<dyn Executable>>,
}

/// Tasks pool
pub struct Pool {
    tasks: HashMap<Id<Slot>, Slot>,
    states: HashMap<TypeId, Vec<Id<Slot>>>,
}

impl Pool {
    /// Constructs new instance of `Pool`
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            states: HashMap::new(),
        }
    }

    /// Returns true if task exists, false otherwise
    pub fn has_task(&self, id: Id<Slot>) -> bool {
        self.tasks.contains_key(&id)
    }

    /// Stores task in the `Pool`
    pub fn store(&mut self, task: Box<dyn Executable>) {
        let task_id = task.id();
        if let Some(slot) = self.tasks.get_mut(&task_id) {
            slot.task = Some(task);
        } else {
            let states = task.states();
            if states.len() == 0 {
                self.states
                    .entry(TypeId::of::<()>())
                    .or_default()
                    .push(task_id);
            } else {
                for state_type_id in task.states() {
                    self.states.entry(*state_type_id).or_default().push(task_id);
                }
            }
            self.tasks.insert(task_id, Slot { task: Some(task) });
        }
    }

    /// Removes task specified by `Id` from the `Pool` and returns it
    pub fn take(&mut self, id: &Id<Slot>) -> Option<Box<dyn Executable>> {
        self.tasks.get_mut(id).and_then(|slot| slot.task.take())
    }

    /// Selects tasks for the state, specified by its type id
    pub fn select_for_state(&self, state: &TypeId) -> Option<&[Id<Slot>]> {
        self.states.get(state).map(|v| v.as_slice())
    }

    /// Resets all tasks
    pub fn reset_tasks(&mut self, queue: &[Id<Slot>]) {
        for id in queue.iter() {
            self.tasks
                .get_mut(id)
                .and_then(|slot| slot.task.as_mut())
                .map(|task| task.reset());
        }
    }

    /// Calculates how many tasks will provide specified output in the queue
    pub fn calculate_context_providers(
        &self,
        queue: &[Id<Slot>],
        output_type_id: std::any::TypeId,
        context: &mut context::Manager,
    ) -> usize {
        let tasks = queue.iter().filter_map(|id| {
            self.tasks
                .get(id)
                .and_then(|slot| slot.task.as_ref())
                .and_then(|task| {
                    if task.output_type_id() == output_type_id {
                        Some(task)
                    } else {
                        None
                    }
                })
        });

        let mut providers = 0;

        for task in tasks {
            let mut p = 1;
            let mut will_run_multiple_times = false;
            for (dep_type_id, dep_type) in task.dependencies().data.iter() {
                match dep_type {
                    context::DependencyType::Any(_) => {
                        let any_providers =
                            self.calculate_context_providers(queue, *dep_type_id, context);
                        if any_providers == 0 {
                            log::warn!(
                                "Task {} dependency on {} could be never satisfied",
                                task.name(),
                                context.output_name(dep_type_id).unwrap_or("UNKNOWN")
                            );
                        } else if any_providers > 1 {
                            if will_run_multiple_times {
                                panic!(
                                    "Task {:?} dependes on more than one multiple Any<T> outputs",
                                    task.name()
                                );
                            }
                            will_run_multiple_times = true;
                            p *= any_providers;
                        }
                    }
                    context::DependencyType::All(_) => {
                        self.calculate_context_providers(queue, *dep_type_id, context);
                    }
                };
            }
            providers += p;
        }
        log::debug!(
            "{} has {} providers",
            context.output_name(&output_type_id).unwrap_or("UNKNOWN"),
            providers
        );
        context.set_output_providers(output_type_id, providers);

        providers
    }
}
