use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::graphics::Extent2D;
use crate::log;
use crate::models::{Image, ImageFormat};

use super::{Asset, ResourceBundle, ResourceLoader, ResourceTarget};

/// Image asset loader
#[derive(Default)]
pub struct ImageLoader;

impl ResourceLoader for ImageLoader {
    fn read(&self, path: &Path, targets: &HashSet<ResourceTarget>) -> ResourceBundle {
        let format = image::ImageFormat::from_path(path)
            .expect("File provided to ImageLoader is of unsupported format");

        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .expect("Could not get file name from its path");

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) => panic!("Could not open image resource file ({path:?}): {err:?}"),
        };
        let metadata = std::fs::metadata(path).expect("Could not read Image file metadata");
        let mut data = vec![0; metadata.len() as usize];
        file.read_exact(&mut data)
            .expect("Could not read Image resource file into buffer");

        let mut bundle: HashMap<ResourceTarget, Option<Box<dyn Asset>>> = targets
            .iter()
            .map(|target| (target.clone(), None))
            .collect::<HashMap<_, _>>();

        let target = ResourceTarget {
            type_id: std::any::TypeId::of::<Image>(),
            name: name.into(),
        };

        if bundle.is_empty() || bundle.contains_key(&target) {
            if let Some(img) = ImageLoader::read_image_buffer(name, &data, format) {
                bundle.insert(target, Some(Box::new(img)));
            }
        }

        ResourceBundle {
            resource: path.into(),
            bundle,
        }
    }
}

impl ImageLoader {
    fn read_image_buffer(
        name: impl Into<String>,
        data: &[u8],
        format: image::ImageFormat,
    ) -> Option<Image> {
        match image::load_from_memory_with_format(data, format) {
            Ok(img) => {
                let resolution = Extent2D {
                    width: img.width(),
                    height: img.height(),
                };
                Some(Image::new(name.into(), resolution, img.into_bytes()))
            }
            Err(e) => {
                log::error!("Could not read image from buffer: {:?}", e);
                None
            }
        }
    }

    pub fn read_buffer(name: impl Into<String>, data: &[u8], format: ImageFormat) -> Option<Image> {
        let format = match format {
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Bmp => image::ImageFormat::Bmp,
        };
        Self::read_image_buffer(name, data, format)
    }
}
