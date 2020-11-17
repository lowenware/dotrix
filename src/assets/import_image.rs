use std::{
    fs::File,
    sync::{Arc, mpsc, Mutex},
};

use super::{
    texture::Texture,
    loader::{ Asset, ImportError, Task, Response },
};


pub fn import_image(
    task: Task,
    sender: &Arc<Mutex<mpsc::Sender<Response>>>
) -> Result<Asset<Texture>, ImportError> {

    let file = File::open(task.path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let texture = import_image_from_bytes(task.path.file_stem(), 
    let decoder = png::Decoder::new(File::open(task.path).unwrap());

    if let Ok((info, mut reader)) = decoder.read_info() {
        let mut texture = Asset {
            name: task.name,
            asset: Texture {
                width: info.width,
                height: info.height,
                depth: 1, 
                data: vec![0; info.buffer_size()],
            }
        };
        reader.next_frame(&mut texture.asset.data).unwrap();
        sender.lock().unwrap().send(Response::Texture(texture)).unwrap();
    }
}

pub fn import_image_from_bytes(
    name: String,
    format: image::ImageFormat,
    data: Vec<u8>,
) -> Result<Asset<Texture>, ImportError> {

    if let Ok(image) = image::load_from_memory_with_format(data.as_slice(), format) {
        let image = image.into_rgba8();

        let (width, height) = image.dimensions();
 
        Asset {
            name,
            asset: Texture {
                width,
                height,
                depth: 1,
                data: image.clone().into_vec(),
            }
        }
    } else {
        Err(ImportError::Unsupported)
    }
}
