use crate::context;
use std::any::{type_name, Any, TypeId};
use std::sync::{Arc, Mutex};

pub type Id = u32;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputChannel {
    /// Send result to pool
    Pool,
    /// Send result to scheduler
    Scheduler,
}

pub trait Task: 'static + Send + Sync + Sized {
    type Context: context::TupleSelector;
    type Output: 'static + Send;

    fn run(&mut self, ctx: Self::Context) -> Self::Output;

    fn output_channel(&self) -> OutputChannel {
        OutputChannel::Pool
    }

    fn boxify(mut self) -> Box<dyn Executable> {
        use context::TupleSelector;
        let task_box: TaskBox<_> = TaskBox {
            id: 0,
            type_id: TypeId::of::<Self>(),
            output_type_id: TypeId::of::<Self::Output>(),
            output_type_name: String::from(type_name::<Self::Output>()),
            name: type_name::<Self>(),
            lock: <Self::Context>::lock(),
            dependencies: <Self::Context>::dependencies(),
            states: <Self::Context>::states(),
            dependencies_state: None,
            output_channel: self.output_channel(),
            run: move |context_manager, dependencies| {
                let task_context = context_manager
                    .lock()
                    .unwrap()
                    .fetch::<Self::Context>(dependencies);
                let task_result = self.run(task_context);
                Box::new(task_result)
            },
        };
        Box::new(task_box)
    }
}

pub struct TaskBox<F>
where
    F: FnMut(
        &Arc<Mutex<context::Manager>>,
        &context::Dependencies,
    ) -> Box<dyn Any + 'static + Send>,
{
    id: Id,
    type_id: TypeId,
    output_type_id: TypeId,
    output_type_name: String,
    name: &'static str,
    lock: context::Lock,
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
    fn id(&self) -> Id;

    /// Get task id
    fn set_id(&mut self, id: Id);

    /// Get task type id
    fn type_id(&self) -> TypeId;

    /// Get type id of result
    fn output_type_id(&self) -> TypeId;

    /// Get type id of result
    fn output_as_str(&self) -> &str;

    /// Get lock for context
    fn lock(&self) -> &context::Lock;

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

    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
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

    fn lock(&self) -> &context::Lock {
        &self.lock
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
