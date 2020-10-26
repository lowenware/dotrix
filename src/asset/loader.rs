use std::{
    sync::{Arc, mpsc, Mutex},
    thread,
};

pub struct Task {
    pub path: String,
}

pub enum Request {
    Import(Box<Task>),
    Terminate,
}

pub struct Loader {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Loader {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Request>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let request = receiver.lock().unwrap().recv().unwrap();
                match request {
                    Request::Import(task) => println!("Loader {} got a task {:?}", id, task.path),
                    Request::Terminate => break,
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }

    pub fn join(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
