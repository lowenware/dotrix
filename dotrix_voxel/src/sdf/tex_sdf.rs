use crate::Grid;
use dotrix_core::renderer::{Renderer, Texture as TextureBuffer};

/// Object to hold the 3D texture containing an Sdf
pub struct TexSdf {
    /// Texture buffer containing a 3d texture
    /// with r channel of the distance anf g channel of the material ID
    pub buffer: TextureBuffer,
}

impl TexSdf {
    pub fn load(&mut self, renderer: &Renderer, grid: &Grid) {
        let pixel_size = 4 * 2;
        let data: Vec<Vec<u8>> = vec![
            0u8;
            pixel_size
                * grid.dimensions[0] as usize
                * grid.dimensions[1] as usize
                * grid.dimensions[2] as usize
        ]
        .chunks(grid.dimensions[0] as usize * grid.dimensions[1] as usize * pixel_size)
        .map(|chunk| chunk.to_vec())
        .collect();

        let slices: Vec<&[u8]> = data.iter().map(|chunk| chunk.as_slice()).collect();

        renderer.load_texture(
            &mut self.buffer,
            grid.dimensions[0],
            grid.dimensions[1],
            slices.as_slice(),
        );
    }
}
