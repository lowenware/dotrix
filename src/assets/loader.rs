use std::{
    fs::File,
    path::PathBuf,
    sync::{Arc, mpsc, Mutex},
    thread,
};

use log::error;

use super::{
    animation::Animation,
    mesh::Mesh,
    skin::Skin,
    texture::Texture,
    load_gltf::load_gltf,
};

pub struct Task {
    pub path: PathBuf,
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
    Animation(Asset<Animation>),
    Texture(Asset<Texture>),
    Mesh(Asset<Mesh>),
    Skin(Asset<Skin>),
}

pub struct Loader {
    thread: Option<thread::JoinHandle<()>>,
}

#[derive(Debug)]
pub enum ImportError {
    Base64Decode(base64::DecodeError),
    FileRead(std::io::Error),
    ImageDecode(image::ImageError),
    GltfDecode(gltf::Error),
    NotImplemented(&'static str, Option<String>),
    Corruption(&'static str),
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
                    Request::Import(task) => {
                        if let Err(e) = import_resource(&task, &sender) {
                            error!("[{}] Resource import from `{:?}` failed: \n\t{:?}",
                                id, task.path, e);
                        }
                    }, 
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

fn import_resource(
    task: &Task,
    sender: &Arc<Mutex<mpsc::Sender<Response>>>,
) -> Result<(), ImportError> {

    use std::io::Read;

    let name = String::from(task.path.file_stem().unwrap().to_str().unwrap());

    let mut file = File::open(&task.path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if let Some(extension) = task.path.extension() {
        let extension = extension.to_str().unwrap();
        match extension {
            "png" | "jpg" | "jpeg" | "bmp" => load_image(
                sender,
                name,
                buffer,
                image::ImageFormat::from_extension(extension).unwrap(),
            ),
            "gltf" | "gltb" => load_gltf(sender, name, buffer, &task.path),
            _ => Err(ImportError::NotImplemented("extension", None)),
        }
    } else {
        Err(ImportError::NotImplemented("file without extension", None))
    }
}

pub fn load_image(
    sender: &Arc<Mutex<mpsc::Sender<Response>>>,
    name: String,
    data: Vec<u8>,
    format: image::ImageFormat,
) -> Result<(), ImportError> {

    let image = image::load_from_memory_with_format(data.as_slice(), format)?;
    let image = image.into_rgba8();

    let (width, height) = image.dimensions();
 
    let texture = Asset {
        name,
        asset: Texture {
            width,
            height,
            depth: 1,
            data: image.into_vec(),
        }
    };
    sender.lock().unwrap().send(Response::Texture(texture)).unwrap();
    Ok(())
}

impl std::error::Error for ImportError {}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::Base64Decode(err) =>
                write!(f, "Can't decode base64 ({:?})", err),
            ImportError::FileRead(err) =>
                write!(f, "Can't read file ({:?})", err),
            ImportError::ImageDecode(err) =>
                write!(f, "Can't decode image ({:?})", err),
            ImportError::GltfDecode(err) =>
                write!(f, "Can't decode GLTF ({:?})", err),
            ImportError::NotImplemented(feature, variant) => 
                write!(f, "Not implemented support for the {:?} ({:?})", feature, variant),
            ImportError::Corruption(err) =>
                write!(f, "File could be corrupted ({:?})", err),
        }
    }
}

impl From<std::io::Error> for ImportError {
    fn from(err: std::io::Error) -> Self {
        ImportError::FileRead(err)
    }
}

impl From<image::ImageError> for ImportError {
    fn from(err: image::ImageError) -> Self {
        ImportError::ImageDecode(err)
    }
}

impl From<base64::DecodeError> for ImportError {
    fn from(err: base64::DecodeError) -> Self {
        ImportError::Base64Decode(err)
    }
}

impl From<gltf::Error> for ImportError {
    fn from(err: gltf::Error) -> Self {
        ImportError::GltfDecode(err)
    }
}
