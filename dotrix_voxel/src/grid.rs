use super::Voxel;
use dotrix_core::{
    renderer::{wgpu, Texture as TextureBuffer},
    Assets, Renderer,
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
    /// 3D Texture buffer
    pub buffer: TextureBuffer,
    /// Tracks if changed since last load
    pub changed: bool,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            dimensions: [16, 16, 16],
            voxel_dimensions: [0.1, 0.1, 0.1],
            position: [0., 0., 0.],
            voxels: vec![Default::default(); 16 * 16 * 16],
            buffer: {
                let mut buffer = TextureBuffer::new_3d("VoxelGrid");
                buffer.format = wgpu::TextureFormat::Rg8Unorm;
                buffer
            },
            changed: false,
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
        Self::flag_changed(self)
    }
    #[must_use]
    pub fn with_voxel_dimensions<T: Into<[f32; 3]>>(mut self, voxel_dimensions: T) -> Self {
        self.voxel_dimensions = voxel_dimensions.into();
        Self::flag_changed(self)
    }
    #[must_use]
    pub fn with_position<T: Into<[f32; 3]>>(mut self, position: T) -> Self {
        self.position = position.into();
        Self::flag_changed(self)
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
        Self::flag_changed(self)
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
        Self::flag_changed(self)
    }

    #[must_use]
    pub fn flag_changed(mut self) -> Self {
        self.changed = true;
        self
    }

    /// Get's the total size of the voxels in all dimensions
    pub fn total_size(&self) -> [f32; 3] {
        [
            self.voxel_dimensions[0] * self.dimensions[0] as f32,
            self.voxel_dimensions[1] * self.dimensions[1] as f32,
            self.voxel_dimensions[2] * self.dimensions[2] as f32,
        ]
    }

    pub fn load(&mut self, renderer: &Renderer, _assets: &mut Assets) {
        if !self.changed && self.buffer.loaded() {
            return;
        }

        let data: Vec<Vec<u8>> = self
            .voxels
            .chunks(self.dimensions[0] as usize * self.dimensions[1] as usize)
            .map(|chunk| {
                chunk
                    .iter()
                    .flat_map(|voxel| [voxel.value, voxel.material])
                    .collect()
            })
            .collect();

        let slices: Vec<&[u8]> = data.iter().map(|chunk| chunk.as_slice()).collect();

        renderer.load_texture(
            &mut self.buffer,
            self.dimensions[0],
            self.dimensions[1],
            slices.as_slice(),
        );

        self.changed = false;
    }

    /// Unloads the [`Texture`] data from the buffer
    pub fn unload(&mut self) {
        self.buffer.unload();
    }
}
