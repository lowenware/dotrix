use std::{
    fs::File,
    sync::{Arc, mpsc, Mutex},
};

use super::{
    texture::Texture,
    loader::{ Asset, Task, Response },
};

pub fn load_png(task: Task, sender: &Arc<Mutex<mpsc::Sender<Response>>>) {
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
    }
}
