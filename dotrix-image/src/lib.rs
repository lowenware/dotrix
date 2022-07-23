use dotrix_assets as assets;
use dotrix_log as log;
use dotrix_types::id;

pub const NAMESPACE: u64 = 0x02;

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
    fn new(name: &str, width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            name: String::from(name),
            width,
            height,
            data,
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
        let mut result = Vec::new();
        if let Ok(img) = image::load_from_memory_with_format(&data, format) {
            let img = img.into_rgba8();
            let (width, height) = img.dimensions();
            let img = Image::new(name, width, height, img.into_vec());
            result.push(Box::new(img) as Box<dyn assets::Asset>);
        } else {
            log::warn!("could not load image from '{:?}'", path);
        }
        result
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
