mod context;
mod scheduler;
mod task;
mod worker;

use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread;

pub use context::{All, Any, Ro, Rw};
pub use task::Task;

/// Task Manager
///
/// Provides control features for multitasking
pub struct TaskManager {
    /// Scheduler thread handle
    scheduler: thread::JoinHandle<()>,
    /// List of workers
    workers: Vec<thread::JoinHandle<()>>,
    /// Receive reports from scheduler
    control_rx: mpsc::Receiver<scheduler::Message>,
    /// Send commands to scheduler
    scheduler_tx: Arc<Mutex<mpsc::Sender<scheduler::Message>>>,
    /// Task Id
    next_task_id: task::Id,
}

/// Tasks Context structure to be used inside of a task as a regular context
pub struct Tasks {
    /// Send commands to scheduler
    scheduler_tx: Arc<Mutex<mpsc::Sender<scheduler::Message>>>,
}

impl TaskManager {
    pub fn new(workers_count: u32) -> Self {
        let context_manager = Arc::new(Mutex::new(context::Manager::new()));
        let (scheduler_tx, scheduler_rx) = mpsc::channel();
        let (worker_tx, worker_rx) = mpsc::channel();
        let (control_tx, control_rx) = mpsc::channel();
        let worker_rx = Arc::new(Mutex::new(worker_rx));
        let scheduler_tx = Arc::new(Mutex::new(scheduler_tx));

        let workers = (0..workers_count)
            .map(|id| {
                worker::spawn(
                    id,
                    Arc::clone(&context_manager),
                    Arc::clone(&worker_rx),
                    Arc::clone(&scheduler_tx),
                )
            })
            .collect::<Vec<_>>();

        let scheduler = scheduler::spawn(context_manager, scheduler_rx, worker_tx, control_tx);

        Self {
            scheduler,
            workers,
            control_rx,
            scheduler_tx,
            next_task_id: 1,
        }
    }

    fn lock_scheduler_tx<'a>(&'a self) -> MutexGuard<'a, mpsc::Sender<scheduler::Message>> {
        self.scheduler_tx.lock().expect("Mutex to be locked")
    }

    pub fn add<T: task::Task>(&mut self, task: T) {
        let tid = self.next_task_id;
        let mut task = task.boxify();
        task.set_id(tid);
        self.lock_scheduler_tx()
            .send(scheduler::Message::Schedule(task))
            .expect("Message to be sent to Scheduler");
        self.next_task_id += 1;
    }

    pub fn store<T: context::Context + Send>(&self, ctx: T) {
        self.lock_scheduler_tx()
            .send(scheduler::Message::Store(
                std::any::TypeId::of::<T>(),
                Box::new(ctx),
            ))
            .expect("Message to be sent to Scheduler");
    }

    pub fn start(&self) {
        self.lock_scheduler_tx()
            .send(scheduler::Message::Provide(
                std::any::TypeId::of::<scheduler::Start>(),
                Box::new(scheduler::Start::default()),
            ))
            .expect("Message to be sent to Scheduler");
    }

    pub fn context(&self) -> Tasks {
        Tasks {
            scheduler_tx: Arc::clone(&self.scheduler_tx),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
