use std::{
    fs::File,
    path::Path,
    sync::{Arc, mpsc, Mutex},
    thread,
};

use super::{
    mesh::Mesh,
    texture::Texture,
};

pub struct Task {
    pub path: String,
    pub name: String,
}

pub struct Asset<T> {
    pub name: String,
    pub asset: T,
}

pub enum Request {
    Import(Task),
    Terminate,
}

pub enum Response {
    Texture(Asset<Texture>),
    Mesh(Asset<Mesh>),
}

pub struct Loader {
    thread: Option<thread::JoinHandle<()>>,
}

impl Loader {
    pub fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Request>>>,
        sender: Arc<Mutex<mpsc::Sender<Response>>>,
    ) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let request = receiver.lock().unwrap().recv().unwrap();
                match request {
                    Request::Import(task) => load_asset(id, task, &sender), 
                    Request::Terminate => break,
                }
            }
        });

        Self {
            thread: Some(thread),
        }
    }

    pub fn join(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

fn load_asset(id: usize, task: Task, sender: &Arc<Mutex<mpsc::Sender<Response>>>) {
    println!("Loader {}: import {}", id, task.path);

    let path = Path::new(&task.path);
    if let Some(extension) = path.extension() {
        let extension = extension.to_str().unwrap();
        match extension {
            "png" => load_png(task, sender),
            "gltf" | "gltb" => load_gltf(task, sender),
            _ => panic!("No loader for `{}`", extension),
        };
    } else {
        panic!("Don't know how to import `{}`", task.path);
    }
}

// TODO: move to load_png.rs
pub fn load_png(task: Task, sender: &Arc<Mutex<mpsc::Sender<Response>>>) {
    println!("Loading png");
    let decoder = png::Decoder::new(File::open(task.path).unwrap());

    if let Ok((info, mut reader)) = decoder.read_info() {
        let mut texture = Asset {
            name: task.name,
            asset: Texture {
                width: info.width,
                height: info.height,
                depth: 1, /*match info.bit_depth {
                    png::BitDepth::One => 1,
                    png::BitDepth::Two => 2,
                    png::BitDepth::Four => 4,
                    png::BitDepth::Eight  => 8,
                    png::BitDepth::Sixteen  => 16,
                },*/
                data: vec![0; info.buffer_size()],
            }
        };
        reader.next_frame(&mut texture.asset.data).unwrap();

        sender.lock().unwrap().send(Response::Texture(texture)).unwrap();
        println!("PNG {}x{}:{:?}, size {}", info.width, info.height, info.bit_depth, info.buffer_size());
    }
}

// TODO: move to load_gltf.rs
pub fn load_gltf(_task: Task, sender: &Arc<Mutex<mpsc::Sender<Response>>>) {
    sender.lock().unwrap().send(Response::Mesh(
        Asset {
            name: String::from("Cube"),
            asset: Mesh::cube()
        }
    )).unwrap();
}
