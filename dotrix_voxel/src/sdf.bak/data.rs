use super::SdfBufferData;
use crate::Grid;
use dotrix_core::{
    assets::Texture,
    renderer::{StorageTextureAccess, TextureBuffer, TextureFormat, TextureUsages, UniformBuffer},
    Pipeline, Renderer, Transform,
};
use dotrix_math::*;

pub struct RenderSdf {
    /// The max iterations for the sdf sphere tracer
    pub max_iterations: u32,
    /// The uniform buffer
    pub sdf_buffer: UniformBuffer,
    /// Pipeline used for the render
    pub pipeline: Pipeline,
    /// Last revision from source grid
    pub revision: u32,
}

impl Default for RenderSdf {
    fn default() -> Self {
        Self {
            max_iterations: 256,
            sdf_buffer: Default::default(),
            compute_pipeline: Default::default(),
            revision: 0,
        }
    }
}

/// Adding this component will compute the sdf from
/// - A voxel [`Grid`]
// TODO: Add compute from mesh
pub struct ComputeSdf {
    /// The 3d texture the density data is read from
    pub value_texture: Option<Texture>,
    /// The 3d texture that the sdf data is stored into
    pub sdf_texture: Option<Texture>,
    /// The pipeline used for the compute
    pub pipeline: Pipeline,
    /// Last revision from source grid
    pub revision: u32,
}

impl Default for ComputeSdf {
    fn default() -> Self {
        Self {
            value_texture: None,
            sdf_texture: None,
            revision: 0,
        }
    }
}

impl RenderSdf {
    /// Loads the buffer that describes how the sdf is rendered
    pub fn load(&mut self, grid: &Grid, transform: &Transform, renderer: &Renderer) {
        // Scale of the cube so that it include the whole size of the grid
        let grid_size = grid.total_size();
        let scale = Mat4::from_nonuniform_scale(grid_size[0], grid_size[1], grid_size[2]);
        let uniform = SdfBufferData {
            cube_transform: scale.into(),
            world_transform: transform.matrix().into(),
            grid_dimensions: grid_size,
            padding: Default::default(),
        };
        renderer.load_uniform_buffer(&mut self.sdf_buffer, bytemuck::cast_slice(&[uniform]));
    }
}

impl ComputeSdf {
    /// Preparet for computation of the sdf
    /// Returns true if sdf calcaultion should be performed
    pub fn load(&mut self, grid: &Grid, renderer: &Renderer) -> bool {
        if grid.revision == self.revision
            && self.value_texture.is_some()
            && self.sdf_texture.is_some()
        {
            return false;
        } else {
            let mut full_reload = false;
            if let Some(texture) = self.value_texture.as_mut() {
                if texture.width != grid.dimensions[0]
                    || texture.height != grid.dimensions[1]
                    || texture.depth != grid.dimensions[2]
                {
                    // Dimensions was changed full reload requried
                    texture.unload();
                    full_reload = true;
                }
            }

            if full_reload || self.sdf_texture.is_none() {
                if let Some(texture) = self.sdf_texture.as_mut() {
                    texture.unload();
                }

                // Get texture from grid
                self.value_texture = Some(grid.gen_texture());

                // Prepare sdf texture
                let count =
                    (grid.dimensions[0] * grid.dimensions[0] * grid.dimensions[0] * 2) as usize;

                self.sdf_texture = Some(Texture {
                    width: grid.dimensions[0],
                    height: grid.dimensions[1],
                    depth: grid.dimensions[2],
                    data: vec![0; count],
                    usages: TextureUsages::create().texture().write(),
                    buffer: TextureBuffer::new(StorageTextureAccess::Read, TextureFormat::rg_u8()),
                    changed: false,
                });
            }

            self.value_texture.as_mut().unwrap().load(renderer);
            self.sdf_texture.as_mut().unwrap().load(renderer);

            self.revision = grid.revision;
            full_reload
        }
    }
}
