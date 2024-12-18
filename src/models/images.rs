use crate::graphics::Extent2D;
use crate::loaders::Asset;

#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
}

// Image in RGBA8 format
#[derive(Debug)]
pub struct Image {
    /// Image name
    name: String,
    /// Image resolution
    resolution: Extent2D,
    /// Raw image data
    data: Vec<u8>,
}

impl Image {
    /// Constructs a new instance of Image
    pub fn new(name: String, resolution: Extent2D, data: Vec<u8>) -> Self {
        Self {
            name,
            resolution,
            data,
        }
    }

    /// Returns image resolution
    pub fn resolution(&self) -> &Extent2D {
        &self.resolution
    }

    /// Returns image bytes
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl Asset for Image {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
