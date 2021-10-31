use dotrix_core::assets::Texture;
use crate::VecXZ;


/// World HeightMap container
pub struct HeightMap {
    size: VecXZ<u32>,
    data: Vec<f32>,
}

impl HeightMap {
    /// Creates new heightmap
    pub fn new(size: VecXZ<u32>) -> Self {
        let capacity = size.x * size.z;
        Self {
            size,
            data: vec![0.0; capacity as usize],
        }
    }

    pub fn size(&self) -> VecXZ<u32> {
        self.size
    }

    /// Returns value for specified X and Z
    pub fn get(&self, point_x: u32, point_z: u32) -> Option<f32> {
        let size_x = self.size.x;
        let size_z = self.size.z;
        if point_x < size_x && point_z < size_z {
            let index = point_z * size_x + point_x;
            Some(self.data[index as usize])
        } else {
            None
        }
    }

    /// Sets value for specified X and Z
    pub fn set(&mut self,  point_x: u32, point_z: u32, value: f32) {
        let size_x = self.size.x;
        let size_z = self.size.z;
        if point_x < size_x && point_z < size_z {
            let index = point_z * size_x + point_x;
            self.data[index as usize] = value;
        }
    }

    /// Adds value for specified X and Z
    pub fn add(&mut self,  point_x: u32, point_z: u32, value: f32) {
        let size_x = self.size.x;
        let size_z = self.size.z;
        if point_x < size_x && point_z < size_z {
            let index = point_z * size_x + point_x;
            let v = self.data[index as usize];
            self.data[index as usize] = v + value;
        }
    }

    /// Resize height map
    pub fn resize(&mut self, new_size_x: usize, new_size_z: usize) {
        let old_size_x = self.size.x as usize;
        let old_size_z = self.size.z as usize;
        let old_length = self.data.len();
        let new_length = new_size_x * new_size_z;
        // Extend the array to move data
        if new_length > old_length {
            self.data.resize(new_length, 0.0);
        }
        for z in (0..new_size_z).rev() {
            for x in (0..new_size_x).rev() {
                let new_index = z * new_size_x + x;
                let old_index = z * old_size_x + x;
                let use_old_value = x < old_size_x && z < old_size_z;
                self.data[new_index] = if use_old_value {
                    self.data[old_index]
                } else {
                    0.0
                }
            }
        }
        // Shrink array if it was necessary
        if new_length < old_length {
            self.data.resize(new_length as usize, 0.0);
        }

        self.size.x = new_size_x as u32;
        self.size.z = new_size_z as u32;
    }

    /// Reset height map
    pub fn reset(&mut self) {
        for value in self.data.iter_mut() {
            *value = 0.0;
        }
    }

    pub fn texture(&self, y_scale: f32) -> Texture {
        let mut min = y_scale;
        let mut max = -y_scale;
        for value in self.data.iter() {
            let v = *value;
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        let delta = max - min;
        let mut data = Vec::with_capacity(self.data.len() * 4);

        for value in self.data.iter() {
            let byte = ((0xFF as f32 * ((value - min) / delta)) as u8) & 0xFF;
            data.push(byte);
            data.push(byte);
            data.push(byte);
            data.push(0xFF);
        }

        Texture {
            data,
            width: self.size.x,
            height: self.size.z,
            ..Default::default()
        }
    }
}

impl From<Texture> for HeightMap {
    fn from(texture: Texture) -> Self {
        let size = VecXZ::new(texture.width, texture.height);
        let capacity = size.x * size.z;
        let bytes_per_pixel = texture.data.len() as u32 / size.x / size.z;
        let mut data = Vec::with_capacity(capacity as usize);
        for x in 0..size.x {
            let index = x * size.z;
            for z in 0..size.z {
                let value = match bytes_per_pixel {
                    1 => texture.data[(index + z) as usize] as f32 / 0xFF as f32,
                    2 => {
                        let index = ((index + z) as usize) * 2;
                        let value = texture.data[index] as u16 |
                            (texture.data[index + 1] as u16) << 8;
                        value as f32 / 0xFFFF as f32
                    },
                    4 => texture.data[4 * (index + z) as usize] as f32 / 0xFF as f32,
                    _ => panic!("Unsupported texture format")
                };
                data.push(value);
            }
        }

        Self {
            size,
            data,
        }
    }
}

/*
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
*/

impl Default for HeightMap {
    fn default() -> Self {
        Self::new(VecXZ::new(1024, 1024))
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
