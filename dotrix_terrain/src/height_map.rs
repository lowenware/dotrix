use dotrix_core::assets::Texture;
use crate::VecXZ;


/// World HeightMap container
pub struct HeightMap {
    size: VecXZ<u32>,
    bytes_per_pixel: u32,
    data: Vec<u8>,
}

impl HeightMap {
    /// Creates new heightmap
    pub fn new(size: VecXZ<u32>, bytes_per_pixel: u32) -> Self {
        let capacity = size.x * size.z * bytes_per_pixel;
        Self {
            size,
            bytes_per_pixel,
            data: vec![0; capacity as usize],
        }
    }

    pub fn size(&self) -> VecXZ<u32> {
        self.size
    }

    /// Returns value for specified X and Z
    pub fn get(&self, point_x: u32, point_z: u32) -> Option<f32> {
        let size_x = self.size.x;
        let size_z = self.size.z;
        let bytes_per_pixel = self.bytes_per_pixel;
        if point_x < size_x && point_z < size_z {
            let mut result: u32 = 0;
            let index = (point_z * size_x + point_x) * bytes_per_pixel;
            for i in 0..bytes_per_pixel {
                if i == 3 { break; }
                // println!("{:#02X}", self.data[(index + i) as usize]);
                result |= (self.data[(index + i) as usize] as u32) << (8 * i);

            }
            Some(result as f32 / 0xFFFFFF as f32)
        } else {
            None
        }
    }

    /// Sets value for specified X and Z
    pub fn set(&mut self,  point_x: u32, point_z: u32, value: u32) {
        let size_x = self.size.x;
        let size_z = self.size.z;
        let bytes_per_pixel = self.bytes_per_pixel;
        if point_x < size_x && point_z < size_z {
            let index = (point_z * size_x + point_x) * bytes_per_pixel;
            for i in 0..bytes_per_pixel {
                self.data[(index + i) as usize] = ((value >> (8 * i)) & 0xFF) as u8;
            }
        }
    }
}

impl From<Texture> for HeightMap {
    fn from(texture: Texture) -> Self {
        let size = VecXZ::new(texture.width, texture.height);
        let bytes_per_pixel = texture.data.len() as u32 / size.x / size.z;
        Self {
            size,
            bytes_per_pixel,
            data: texture.data
        }
    }
}

impl From<HeightMap> for Texture {
    fn from(height_map: HeightMap) -> Self {
        Texture {
            width: height_map.size.x,
            height: height_map.size.z,
            data: height_map.data,
            ..Default::default()
        }
    }
}

impl Default for HeightMap {
    fn default() -> Self {
        Self::new(VecXZ::new(1024, 1024), 2)
    }
}

/*
/// Trait for the terrain heights source
pub trait GetHeight: Any + Sync + Send {
    /// Returns Y axis value for specified X and Z pair
    fn value(&self, x: usize, z: usize) -> f32;
    /// Returns number of values per map side
    fn size(&self) -> usize;
}


impl dyn GetHeight {
    /// Casts down the reference
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&*(self as *const dyn GetHeight as *const T)) }
        } else {
            None
        }
    }

    /// Casts down the mutable reference
    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&mut *(self as *mut dyn GetHeight as *mut T)) }
        } else {
            None
        }
    }

    /// Checks if the reference is of specific type
    #[inline]
    fn is<T: Any>(&self) -> bool {
        std::any::TypeId::of::<T>() == self.type_id()
    }
}
*/
