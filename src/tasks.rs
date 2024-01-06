mod context;
mod scheduler;
mod task;
mod worker;

use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread;

use crate::log;
use crate::utils::Id;

use context::Context;

pub use context::{All, Any, Mut, Ref, State, Take};
pub use task::{Output, OutputChannel, Task};

/// Dotrix Task Manager
///
/// Provides control features for multitasking
pub struct TaskManager {
    /// Scheduler thread handle
    scheduler: Option<thread::JoinHandle<()>>,
    /// List of workers
    workers: Vec<thread::JoinHandle<()>>,
    /// Receive reports from scheduler
    control_rx: mpsc::Receiver<scheduler::Message>,
    /// Send commands to scheduler
    scheduler_tx: Arc<Mutex<mpsc::Sender<scheduler::Message>>>,
    /// Context manager
    context: Arc<Mutex<context::Manager>>,
}

pub struct Scheduler<'a> {
    guard: MutexGuard<'a, mpsc::Sender<scheduler::Message>>,
}

impl<'a> Scheduler<'a> {
    /// Add task to the scheduler
    pub fn add_task<T: task::Task>(&self, task: T) -> Id<task::Slot> {
        let id = Id::new();
        let task = task.boxify(id);
        self.guard
            .send(scheduler::Message::Schedule(task))
            .expect("Message to be sent to Scheduler");
        id
    }

    /// Add global data to the context
    pub fn add_context<T: context::Context + Send>(&self, ctx: T) {
        self.guard
            .send(scheduler::Message::Store(
                std::any::TypeId::of::<T>(),
                Box::new(ctx),
            ))
            .expect("Message to be sent to Scheduler");
    }
}

impl TaskManager {
    /// Creates `TaskManager` instance
    pub fn new<T: Context>(workers_count: u32) -> Self {
        log::info!("Initializing manager with {} workers", workers_count);
        let context = Arc::new(Mutex::new(context::Manager::new()));
        let (scheduler_tx, scheduler_rx) = mpsc::channel();
        let (worker_tx, worker_rx) = mpsc::channel();
        let (control_tx, control_rx) = mpsc::channel();
        let worker_rx = Arc::new(Mutex::new(worker_rx));
        let scheduler_tx = Arc::new(Mutex::new(scheduler_tx));

        let workers = (0..workers_count)
            .map(|id| {
                worker::spawn(
                    id,
                    Arc::clone(&context),
                    Arc::clone(&worker_rx),
                    Arc::clone(&scheduler_tx),
                )
            })
            .collect::<Vec<_>>();

        let scheduler =
            scheduler::spawn::<T>(Arc::clone(&context), scheduler_rx, worker_tx, control_tx);

        Self {
            scheduler: Some(scheduler),
            workers,
            control_rx,
            scheduler_tx,
            context,
        }
    }

    fn lock_scheduler_tx<'a>(&'a self) -> MutexGuard<'a, mpsc::Sender<scheduler::Message>> {
        self.scheduler_tx.lock().expect("Mutex to be locked")
    }

    pub fn scheduler<'a>(&'a self) -> Scheduler<'a> {
        Scheduler {
            guard: self.lock_scheduler_tx(),
        }
    }

    // / Add task to the scheduler
    //pub fn schedule<T: task::Task>(&self, task: T) -> Id<task::Slot> {
    //    let id = Id::random();
    //    let task = task.boxify(id);
    //    self.lock_scheduler_tx()
    //       .send(scheduler::Message::Schedule(task))
    //        .expect("Message to be sent to Scheduler");
    //    id
    //}

    // / Add global data to the context
    // pub fn store<T: context::Context + Send>(&self, ctx: T) {
    //    self.lock_scheduler_tx()
    //        .send(scheduler::Message::Store(
    //            std::any::TypeId::of::<T>(),
    //            Box::new(ctx),
    //        ))
    //        .expect("Message to be sent to Scheduler");
    // }

    /// Remove dta from global context
    pub fn remove_global_context<T: context::Context + Send>(&self) -> Option<T> {
        self.context
            .lock()
            .expect("Mutex to be locked")
            .remove_global::<T>()
    }

    /// Register output data type
    pub fn register<T: context::Context + Send>(&self, providers: usize) {
        self.lock_scheduler_tx()
            .send(scheduler::Message::Register(
                std::any::TypeId::of::<T>(),
                std::any::type_name::<T>().into(),
                providers,
            ))
            .expect("Message to be sent to Scheduler");
    }

    /// Provide output data
    pub fn provide<T: context::Context + Send>(&self, data: T) {
        self.lock_scheduler_tx()
            .send(scheduler::Message::Provide(
                std::any::TypeId::of::<T>(),
                Box::new(data),
            ))
            .expect("Message to be sent to Scheduler");
    }

    /// Executes tasks cycle
    pub fn run(&self) {
        self.provide(scheduler::Loop::default());
    }

    /// Waits until data of specified type provided
    pub fn wait_for<T: std::any::Any>(&self) -> T {
        loop {
            let message = self.control_rx.recv().expect("Message to be received");
            if let scheduler::Message::Provide(_type_id, data) = message {
                if let Ok(downcasted_data) = data.downcast::<T>() {
                    return *downcasted_data;
                }
            }
        }
    }

    /// Waits for a message from the control channel
    pub fn wait_message(&self) -> Box<dyn std::any::Any> {
        loop {
            let message = self.control_rx.recv().expect("Message to be received");
            if let scheduler::Message::Provide(_type_id, data) = message {
                return data;
            }
        }
    }
}

impl Drop for TaskManager {
    fn drop(&mut self) {
        let workers = self.workers.len();
        // kill workers
        self.lock_scheduler_tx()
            .send(scheduler::Message::Kill(workers))
            .expect("Message to be sent to Scheduler");

        for worker in self.workers.drain(0..) {
            worker.join().ok();
        }

        // kill scheduler
        self.lock_scheduler_tx()
            .send(scheduler::Message::Kill(0))
            .expect("Message to be sent to Scheduler");

        if let Some(scheduler) = self.scheduler.take() {
            scheduler.join().ok();
        }
    }
}

// pub trait Extension: 'static + Send {
// Setup extension
//    fn load(&self, manager: &Manager);
//}
