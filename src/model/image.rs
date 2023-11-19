use dotrix_assets as assets;
use dotrix_log as log;
use dotrix_types::id;

pub const NAMESPACE: u64 = 0x02;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Png,
    Jpeg,
    Bmp,
}

// Image in RGBA8 format
pub struct Image {
    /// Image name
    pub name: String,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Raw image data
    pub data: Vec<u8>,
}

impl Image {
    /// Constructs a new instance of Image
    pub fn new(name: String, width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            name,
            width,
            height,
            data,
        }
    }

    pub fn from_buffer_as(name: String, data: &[u8], format: Format) -> Option<Self> {
        let format = match format {
            Format::Png => image::ImageFormat::Png,
            Format::Jpeg => image::ImageFormat::Jpeg,
            Format::Bmp => image::ImageFormat::Bmp,
        };
        match image::load_from_memory_with_format(&data, format) {
            Ok(img) => {
                let img = img.into_rgba8();
                let (width, height) = img.dimensions();
                Some(Image::new(name, width, height, img.into_vec()))
            }
            Err(e) => {
                log::error!("Could not read image from buffer: {:?}", e);
                None
            }
        }
    }
}

impl id::NameSpace for Image {
    fn namespace() -> u64 {
        assets::NAMESPACE | NAMESPACE
    }
}

impl assets::Asset for Image {
    fn name(&self) -> &str {
        &self.name
    }

    fn namespace(&self) -> u64 {
        <Self as id::NameSpace>::namespace()
    }
}

/// Image asset loader
#[derive(Default)]
pub struct Loader;

impl assets::Loader for Loader {
    fn can_load(&self, path: &std::path::Path) -> bool {
        image::ImageFormat::from_path(path).is_ok()
    }

    fn load(&self, path: &std::path::Path, data: Vec<u8>) -> Vec<Box<dyn assets::Asset>> {
        let format = image::ImageFormat::from_path(path).unwrap();
        let name = path.file_stem().map(|n| n.to_str().unwrap()).unwrap();

        if let Ok(img) = image::load_from_memory_with_format(&data, format) {
            let img = img.into_rgba8();
            let (width, height) = img.dimensions();
            let img = Image::new(name.into(), width, height, img.into_vec());
            return vec![Box::new(img) as Box<dyn assets::Asset>];
        } else {
            log::warn!("could not load image from '{:?}'", path);
        }
        vec![]
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
