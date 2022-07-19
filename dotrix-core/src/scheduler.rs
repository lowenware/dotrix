use crate::{context, task};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// Local synonym for boxified task
pub type Task = Box<dyn task::Executable>;

// TODO: implement debug
/// Message to scheduler from main process or from worker
pub enum Message {
    /// Schedule a task
    Schedule(Task),
    /// Unschedule existing task by id
    Unschedule(task::Id),
    /// Complete task report
    Complete(Task, Box<dyn Any + 'static + Send>),
    /// Store a new context
    Store(TypeId, Box<dyn Any + 'static + Send>),
    /// Discard previously stored context
    Discard(TypeId),
    /// Provide dependency data for tasks
    Provide(TypeId, Box<dyn Any + 'static + Send>),
    /// Subscribe to data from worker
    Subscribe(TypeId),
    /// Unsubscribe from notifications
    Unsubscribe(TypeId),
    /// Kill Signal
    Kill,
}

#[derive(Default)]
pub struct Start {}

#[derive(Default)]
pub struct Done {}

pub struct TaskSlot {
    pub task: Option<Task>,
    pub executions_count: u64,
}

/// Launches operator thread, that schedules tasks, holds up context and communicates back
/// to main process
///
/// input_rx -> recieve requests from main process and workers
/// worker_tx -> send commands to workers
/// control_tx -> response to control requests to main process
pub fn spawn(
    context_manager: Arc<Mutex<context::Manager>>,
    input_rx: mpsc::Receiver<Message>,
    worker_tx: mpsc::Sender<Message>,
    control_tx: mpsc::Sender<Message>,
) -> thread::JoinHandle<()> {
    // tasks pool
    let name = String::from("dotrix::scheduler");
    // lock manager controls access to context
    let mut lock_manager = context::LockManager::new();
    // all tasks of the application
    let mut pool = HashMap::<task::Id, TaskSlot>::new();
    // tasks selected for execution
    let mut queue = Vec::<task::Id>::new();
    // flag controls change of the tasks graph
    let mut tasks_graph_changed = true;

    context_manager
        .lock()
        .expect("Mutex to be locked")
        .register(std::any::TypeId::of::<Start>());

    thread::Builder::new()
        .name(name)
        .spawn(move || {
            let mut lock_for_input = false;
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
                            tasks_graph_changed = schedule_task(task, &mut pool, &mut lock_manager);
                        }
                        Message::Unschedule(task_id) => {
                            pool.remove(&task_id);
                            tasks_graph_changed = true;
                        }
                        Message::Complete(task, data) => {
                            let type_id = task.provides();
                            schedule_task(task, &mut pool, &mut lock_manager);
                            if TypeId::of::<Done>() == type_id {
                                control_tx.send(Message::Provide(type_id, data)).ok();
                            } else {
                                store_data(
                                    type_id,
                                    data,
                                    &context_manager,
                                    &mut pool,
                                    &mut queue,
                                    &mut tasks_graph_changed,
                                );
                            }
                        }
                        Message::Store(type_id, ctx) => {
                            context_manager.lock().unwrap().store(type_id, ctx);
                        }
                        Message::Discard(type_id) => {
                            context_manager.lock().unwrap().discard(type_id);
                        }
                        Message::Provide(type_id, data) => {
                            store_data(
                                type_id,
                                data,
                                &context_manager,
                                &mut pool,
                                &mut queue,
                                &mut tasks_graph_changed,
                            );
                        }
                        Message::Subscribe(type_id) => {
                            context_manager.lock().unwrap().subscribe(type_id);
                        }
                        Message::Unsubscribe(type_id) => {
                            context_manager.lock().unwrap().unsubscribe(type_id);
                        }
                        Message::Kill => {
                            panic!("Message::Kill is not implemented");
                        }
                    }
                    lock_for_input = false;
                    // There could be some other commands, that must be processed first, before
                    // we schedule new tasks
                    continue;
                }

                // execute tasks
                let mut index = 0;
                let mut stop_index = queue.len();
                // let instant = std::time::Instant::now();
                while index < stop_index {
                    let task_id = queue[index];
                    if let Some(slot) = pool.get_mut(&task_id) {
                        if let Some(mut task) = slot.task.take() {
                            if !task.is_scheduled() {
                                if let Some(dependencies_state) = context_manager
                                    .lock()
                                    .unwrap()
                                    .match_dependencies(task.dependencies(), false)
                                {
                                    task.schedule_with(dependencies_state);
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
                            slot.task = Some(task);
                        }
                    }
                    index += 1;
                }
                lock_for_input = true;
            }
        })
        .expect("Thread to be spawned")
}

/// Schedules task and returns true if task is new, false if task was accepted from worker
fn schedule_task(
    task: Task,
    pool: &mut HashMap<task::Id, TaskSlot>,
    lock_manager: &mut context::LockManager,
) -> bool {
    let task_id = task.id();
    if let Some(mut slot) = pool.get_mut(&task_id) {
        lock_manager.unlock(task.lock());
        slot.task = Some(task);
        slot.executions_count += 1;
        return false;
    }
    pool.insert(
        task_id,
        TaskSlot {
            task: Some(task),
            executions_count: 0,
        },
    );
    true
}

fn store_data(
    type_id: TypeId,
    data: Box<dyn Any + 'static + Send>,
    context_manager: &Arc<Mutex<context::Manager>>,
    pool: &mut HashMap<task::Id, TaskSlot>,
    queue: &mut Vec<task::Id>,
    tasks_graph_changed: &mut bool,
) {
    let mut ctx = context_manager.lock().expect("Mutex to be locked");
    if type_id == TypeId::of::<Start>() {
        ctx.reset_data(*tasks_graph_changed);
        ctx.provide(type_id, data);
        ctx.apply_states_changes();
        // clear queue
        queue.clear();

        // add tasks to queue for the new cycle
        for (tid, slot) in pool.iter_mut() {
            if let Some(task) = slot.task.as_mut() {
                if ctx.match_states(task.states()) {
                    task.reset();
                    queue.push(*tid);
                }
            }
        }

        if *tasks_graph_changed {
            ctx.rebuild_graph(&pool, &queue);
            *tasks_graph_changed = false;
        }
    } else {
        ctx.provide(type_id, data);
    }
}
