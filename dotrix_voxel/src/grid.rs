use super::Voxel;
use dotrix_core::{
    renderer::{wgpu, Texture as TextureBuffer},
    Assets, Renderer,
};

const DEFAULT_DIM: usize = 3;

/// A grid of voxels
pub struct Grid {
    /// The number of voxels per dimension
    /// This is not public to restric resizing
    /// which needs a full recreate and rebind
    /// of all of it's 3d textures if changed
    /// Instead it can only be resized with
    /// `[with_dimensions]` which takes self
    dimensions: [u32; 3],
    /// The voxels
    voxels: Vec<Voxel>,
    /// 3D Texture buffer
    buffer: TextureBuffer,
    /// Tracks if changed by incremented a revision number on each change
    revision: u32,
    /// Revision number of last load
    /// Setting this to `None` will force it to reload
    last_update: Option<u32>,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            dimensions: [DEFAULT_DIM as u32, DEFAULT_DIM as u32, DEFAULT_DIM as u32],
            voxels: vec![Default::default(); DEFAULT_DIM * DEFAULT_DIM * DEFAULT_DIM],
            buffer: {
                let mut buffer = TextureBuffer::new_3d("VoxelGrid");
                buffer.format = wgpu::TextureFormat::Rg8Uint;
                buffer
            },
            revision: 0,
            last_update: None,
        }
    }
}

impl Grid {
    /// Start building a grid
    pub fn build() -> Self {
        Default::default()
    }
    #[must_use]
    /// Build the grid with new dimensions, default values will fill the new voxels
    pub fn with_dimensions<T: Into<[u32; 3]>>(mut self, dimensions: T) -> Self {
        let dimensions: [u32; 3] = dimensions.into();
        self.dimensions = dimensions;
        let count: usize = (dimensions[0] * dimensions[1] * dimensions[2]) as usize;
        // Resize number of voxels to match
        self.voxels.resize(count, Default::default());
        // Must recreate the binding texture
        // Shader must rebind to see the update
        self.buffer = {
            let mut buffer = TextureBuffer::new_3d("VoxelGrid");
            buffer.format = wgpu::TextureFormat::Rg8Uint;
            buffer
        };
        Self::flag_changed(self)
    }
    #[must_use]
    /// Build the grid with these values for the voxel
    pub fn with_values<T: AsRef<[u8]>>(mut self, values: T) -> Self {
        self.set_values(values);
        self
    }
    /// Set the values of the voxel, Extra values are ignored
    pub fn set_values<T: AsRef<[u8]>>(&mut self, values: T) {
        let input: &[u8] = values.as_ref();
        let count: usize = (self.dimensions[0] * self.dimensions[1] * self.dimensions[2]) as usize;

        let slice_len = std::cmp::min(input.len(), count);

        // If they provide too much or too little values it is silently ignored
        // TODO: Should we panic?
        self.voxels[0..slice_len]
            .iter_mut()
            .enumerate()
            .for_each(|(i, v)| (v).value = input[i]);
        self.set_changed();
    }
    #[must_use]
    /// Build the grid with these material values
    pub fn with_materials<T: AsRef<[u8]>>(mut self, values: T) -> Self {
        self.set_materials(values);
        self
    }
    /// Set the material values
    pub fn set_materials<T: AsRef<[u8]>>(&mut self, values: T) {
        let input: &[u8] = values.as_ref();
        let count: usize = (self.dimensions[0] * self.dimensions[1] * self.dimensions[2]) as usize;

        let slice_len = std::cmp::min(input.len(), count);

        // If they provide too much or too little values it is silently ignored
        // TODO: Should we panic?
        self.voxels[0..slice_len]
            .iter_mut()
            .enumerate()
            .for_each(|(i, v)| (v).material = input[i]);
        self.set_changed();
    }

    #[must_use]
    /// Build the grid with it flagged as changed
    pub fn flag_changed(mut self) -> Self {
        self.set_changed();
        self
    }
    /// Set the grid as changed
    pub fn set_changed(&mut self) {
        self.revision += 1;
    }

    /// Get the current revision
    /// This is incremented on each change of the grid
    /// that requires a reload
    pub fn get_revision(&self) -> u32 {
        self.revision
    }

    /// The 3DTexture buffer, must first be loaded with [`load`]
    pub fn get_buffer(&self) -> &TextureBuffer {
        &self.buffer
    }

    /// The number of voxels in each dimension
    pub fn get_dimensions(&self) -> &[u32; 3] {
        &self.dimensions
    }

    /// Same as `[get_dimensions]` but as f32 (convenince method)
    pub fn get_size(&self) -> [f32; 3] {
        [
            self.dimensions[0] as f32,
            self.dimensions[1] as f32,
            self.dimensions[2] as f32,
        ]
    }

    /// Load the grid into a 3DTexture
    pub fn load(&mut self, renderer: &Renderer, _assets: &Assets) {
        if let Some(last_update) = self.last_update {
            if last_update == self.revision && self.buffer.loaded() {
                return;
            }
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

        renderer.update_or_load_texture(
            &mut self.buffer,
            self.dimensions[0],
            self.dimensions[1],
            slices.as_slice(),
        );

        self.last_update = Some(self.revision);
    }

    /// Unloads the [`Grid`] data from the GPU
    pub fn unload(&mut self) {
        self.buffer.unload();
    }
}
