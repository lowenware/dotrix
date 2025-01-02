use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use super::{context, scheduler};
use crate::log;

pub fn spawn(
    id: u32,
    context_manager: Arc<Mutex<context::Manager>>,
    rx: Arc<Mutex<mpsc::Receiver<scheduler::Message>>>,
    tx: Arc<Mutex<mpsc::Sender<scheduler::Message>>>,
) -> thread::JoinHandle<()> {
    let name = format!("dotrix::worker[{}]", id + 1);
    thread::Builder::new()
        .name(name.clone())
        .spawn(move || {
            log::info!("started: {}", name);
            loop {
                let message = rx.lock().unwrap().recv().unwrap();
                match message {
                    scheduler::Message::Schedule(mut task) => {
                        let result = task.run(&context_manager);
                        let response = tx.lock().expect("Mutex to be locked");
                        response.send(scheduler::Message::Output(task, result)).ok();
                    }
                    scheduler::Message::Kill(index) => {
                        log::info!("worker[{id}] goes off by command #{index}");
                        break;
                    }
                    // TODO: implement Debug trait for message
                    _ => println!("{name}: message ignored"),
                };
            }
        })
        .expect("Thread to be spawned")
}
