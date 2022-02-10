use super::Voxel;
use dotrix_core::{
    assets::Texture,
    renderer::{StorageTextureAccess, TextureBuffer, TextureFormat, TextureUsages},
};

/// A grid of voxels
pub struct Grid {
    /// The number of voxels per dimension
    pub dimensions: [u32; 3],
    /// The physical size of a single voxel
    pub voxel_dimensions: [f32; 3],
    /// The position of the grid center
    pub position: [f32; 3],
    /// The voxels
    pub voxels: Vec<Voxel>,
    /// Revision should be updated on each change
    pub revision: u32,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            dimensions: [16, 16, 16],
            voxel_dimensions: [0.1, 0.1, 0.1],
            position: [0., 0., 0.],
            voxels: vec![Default::default(); 16 * 16 * 16],
            revision: 1,
        }
    }
}

impl Grid {
    pub fn build() -> Self {
        Default::default()
    }
    #[must_use]
    pub fn with_dimensions<T: Into<[u32; 3]>>(mut self, dimensions: T) -> Self {
        let dimensions: [u32; 3] = dimensions.into();
        self.dimensions = dimensions;
        let count: usize = (dimensions[0] * dimensions[1] * dimensions[2]) as usize;
        // Resize number of voxels to match
        self.voxels.resize(count, Default::default());
        self
    }
    #[must_use]
    pub fn with_voxel_dimensions<T: Into<[f32; 3]>>(mut self, voxel_dimensions: T) -> Self {
        self.voxel_dimensions = voxel_dimensions.into();
        self
    }
    #[must_use]
    pub fn with_position<T: Into<[f32; 3]>>(mut self, position: T) -> Self {
        self.position = position.into();
        self
    }
    #[must_use]
    pub fn with_values<T: AsRef<[u8]>>(mut self, values: T) -> Self {
        let input: &[u8] = values.as_ref();
        let count: usize = (self.dimensions[0] * self.dimensions[1] * self.dimensions[2]) as usize;

        let slice_len = std::cmp::min(input.len(), count);

        // If they provide too much or too little values it is silently ignored
        // TODO: Should we panic?
        self.voxels[0..slice_len]
            .iter_mut()
            .enumerate()
            .for_each(|(i, v)| (v).value = input[i]);
        self
    }
    #[must_use]
    pub fn with_materials<T: AsRef<[u8]>>(mut self, values: T) -> Self {
        let input: &[u8] = values.as_ref();
        let count: usize = (self.dimensions[0] * self.dimensions[1] * self.dimensions[2]) as usize;

        let slice_len = std::cmp::min(input.len(), count);

        // If they provide too much or too little values it is silently ignored
        // TODO: Should we panic?
        self.voxels[0..slice_len]
            .iter_mut()
            .enumerate()
            .for_each(|(i, v)| (v).material = input[i]);
        self
    }

    /// Create a 3d texture of this grid
    ///
    /// r channel will contain the value
    /// g channel will contain the material
    pub fn gen_texture(&self) -> Texture {
        Texture {
            width: self.dimensions[0],
            height: self.dimensions[1],
            depth: self.dimensions[2],
            data: self
                .voxels
                .iter()
                .flat_map(|v| [v.value, v.material])
                .collect(),
            usages: TextureUsages::create().texture().write(),
            buffer: TextureBuffer::new(StorageTextureAccess::Read, TextureFormat::rg_u8()),
            changed: false,
        }
    }

    /// Get's the total size of the voxels in all dimensions
    pub fn total_size(&self) -> [f32; 3] {
        [
            self.voxel_dimensions[0] * self.dimensions[0] as f32,
            self.voxel_dimensions[1] * self.dimensions[1] as f32,
            self.voxel_dimensions[2] * self.dimensions[2] as f32,
        ]
    }
}
