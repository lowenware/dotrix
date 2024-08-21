use std::any::{Any, TypeId};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::utils::{Id, TypeLock};

use super::{context, task};

/// Local synonym for boxified task
pub type Task = Box<dyn task::Executable>;

/// Message to scheduler from main process or from worker
pub enum Message {
    /// Schedule a task
    Schedule(Task),
    /// Complete task report
    Output(Task, Box<dyn Any + 'static + Send>),
    /// Store a new global context
    Store(TypeId, Box<dyn Any + 'static + Send>),
    /// Register dependency type (with Name)
    Register(TypeId, String, usize),
    /// Provide dependency data for tasks
    Provide(TypeId, Box<dyn Any + 'static + Send>),
    /// Kill Signal
    Kill(usize),
}

/// Scheduling loop control structure
#[derive(Default)]
pub struct Loop;

/// Launches operator thread, that schedules tasks, holds up context and communicates back
/// to main process
///
/// input_rx -> recieve requests from main process and workers
/// worker_tx -> send commands to workers
/// control_tx -> response to control requests to main process
pub fn spawn<T: context::Context>(
    context_manager: Arc<Mutex<context::Manager>>,
    input_rx: mpsc::Receiver<Message>,
    worker_tx: mpsc::Sender<Message>,
    control_tx: mpsc::Sender<Message>,
) -> thread::JoinHandle<()> {
    // tasks pool
    let name = String::from("dotrix::scheduler");
    // lock manager controls access to context
    let mut lock_manager = TypeLock::new();
    // all tasks of the application
    let mut pool = task::Pool::new();
    // tasks selected for execution
    let mut queue: Vec<Id<task::Slot>> = vec![];
    // flag controls change of the tasks graph
    let mut tasks_graph_changed = true;

    context_manager
        .lock()
        .expect("Mutex to be locked")
        .register(
            std::any::TypeId::of::<Loop>(),
            std::any::type_name::<Loop>().into(),
            1,
            false,
        );

    thread::Builder::new()
        .name(name)
        .spawn(move || {
            let mut lock_for_input = false;
            let mut restart_queue = false;
            let mut queue_executed = true;
            loop {
                let mut command = if lock_for_input {
                    // There is nothing else to do, except for waiting
                    Some(input_rx.recv().expect("Message to be received"))
                } else {
                    input_rx.try_recv().map(|c| Some(c)).unwrap_or(None)
                };
                if let Some(command) = command.take() {
                    match command {
                        Message::Schedule(task) => {
                            context_manager.lock().unwrap().register_provider(&task);
                            pool.store(task);
                            tasks_graph_changed = true;
                        }
                        Message::Output(task, data) => {
                            let type_id = task.output_type_id();
                            let output_channel = task.output_channel();
                            lock_manager.unlock(task.lock());
                            pool.store(task);

                            match output_channel {
                                task::OutputChannel::Pool => {
                                    context_manager.lock().unwrap().provide(type_id, data);
                                }
                                task::OutputChannel::Scheduler => {
                                    control_tx.send(Message::Provide(type_id, data)).ok();
                                }
                            };

                            if type_id == TypeId::of::<T>() {
                                queue_executed = true;
                            }
                        }
                        Message::Store(type_id, ctx) => {
                            context_manager.lock().unwrap().store_boxed(type_id, ctx);
                        }
                        Message::Register(type_id, name, providers) => {
                            context_manager
                                .lock()
                                .unwrap()
                                .register(type_id, name, providers, true);
                        }

                        Message::Provide(type_id, data) => {
                            if type_id == TypeId::of::<Loop>() {
                                restart_queue = true;
                            } else {
                                context_manager.lock().unwrap().provide(type_id, data);
                            }
                        }
                        Message::Kill(workers) => {
                            for i in 0..workers {
                                log::info!("sending kill comand to worker {i}");
                                worker_tx.send(Message::Kill(i)).ok();
                            }
                            if workers == 0 {
                                return;
                            }
                        }
                    }
                    lock_for_input = false;
                    // There could be some other commands, that must be processed first, before
                    // we schedule new tasks
                    continue;
                }

                if restart_queue {
                    log::debug!("restart queue(queue_executed: {}", queue_executed);
                    if queue_executed {
                        let mut ctx = context_manager.lock().expect("Mutex to be locked");
                        ctx.reset_data(tasks_graph_changed);
                        ctx.apply_states_changes();
                        queue.clear();
                        ctx.provide(TypeId::of::<Loop>(), Box::new(Loop::default()));

                        let default_state = TypeId::of::<()>();
                        let current_state = ctx.current_state();
                        if current_state != default_state {
                            if let Some(tasks) = pool.select_for_state(&default_state) {
                                queue.extend_from_slice(tasks);
                            }
                        }
                        if let Some(tasks) = pool.select_for_state(&current_state) {
                            queue.extend_from_slice(tasks);
                        }

                        pool.reset_tasks(&queue);

                        if tasks_graph_changed {
                            unsafe {
                                // TODO: move completely to the Pool
                                ctx.calculate_providers::<T>(&pool, &queue);
                            }
                            tasks_graph_changed = false;
                        }
                        queue_executed = false;
                        restart_queue = false;
                    }
                }

                // execute tasks
                let mut index = 0;
                let mut stop_index = queue.len();
                // let instant = std::time::Instant::now();
                while index < stop_index {
                    let task_id = queue[index];
                    if let Some(mut task) = pool.take(&task_id) {
                        log::debug!("task({}): begin control", task.name());
                        if !task.is_scheduled() {
                            log::debug!("task({}): not scheduled yet", task.name());
                            if let Some(dependencies_state) = context_manager
                                .lock()
                                .unwrap()
                                .match_dependencies(task.dependencies())
                            {
                                log::debug!("task({}): to be scheduled", task.name());
                                task.schedule_with(dependencies_state);
                            } else {
                                log::debug!("task({}): dependencies are not sattisfied", task.name());
                            }
                        }

                        // get dependencies
                        if task.is_scheduled() {
                            if lock_manager.lock(task.lock()) {
                                // move to the end of queue
                                queue.remove(index);
                                queue.push(task_id);
                                worker_tx.send(Message::Schedule(task)).ok();
                                stop_index -= 1;
                                continue;
                            }
                        }
                        // postpone execution
                        pool.store(task);
                    }
                    index += 1;
                }
                lock_for_input = true;
            }
        })
        .expect("Thread to be spawned")
}
