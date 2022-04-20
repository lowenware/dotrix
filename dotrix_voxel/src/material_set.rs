//! Set of materials used for voxels
//!
//! Each voxel has a material ID, which corresponds to a
//! material in this set
//!
//! All material textures must have the same size
//!

use dotrix_core::{
    assets::Texture,
    renderer::{Buffer, Texture as TextureBuffer},
    Assets, Id, Renderer,
};
use dotrix_pbr::Material;

use std::collections::HashMap;

pub struct MaterialSet {
    materials: HashMap<u8, Material>,
    last_num_textures: u32,
    texture_buffer: TextureBuffer,
    indicies_buffer: Buffer,
}

impl Default for MaterialSet {
    fn default() -> Self {
        Self {
            materials: Default::default(),
            last_num_textures: 0,

            texture_buffer: TextureBuffer::new_array("MaterialSet")
                .use_as_texture()
                .rgba_u8norm_srgb(),
            indicies_buffer: Buffer::uniform("MaterialSetIndicies"),
        }
    }
}

impl MaterialSet {
    pub fn set_material(&mut self, material_id: u8, material: Material) {
        self.materials.insert(material_id, material);
    }

    /// Returns true if a full rebind is required
    /// returns false if rebind is not required, but may still update textures
    /// by replacing current textures.
    pub fn load(&mut self, renderer: &Renderer, assets: &mut Assets) -> bool {
        let mut result = false;
        let num_texs = self.materials.values().fold(0, |mut acc, mat| {
            if !mat.texture.is_null() {
                acc += 1;
            }
            if !mat.roughness_texture.is_null() {
                acc += 1;
            }
            if !mat.metallic_texture.is_null() {
                acc += 1;
            }
            if !mat.ao_texture.is_null() {
                acc += 1;
            }
            if !mat.normal_texture.is_null() {
                acc += 1;
            }
            acc
        });
        if num_texs != self.last_num_textures {
            // Full reload and bind required
            // because number of textures was changed
            result = true;
        }

        if result {
            self.texture_buffer.unload();
        }

        let number_of_materials = 256;
        let number_of_textures_per_material = 5;
        let number_of_indicies = number_of_materials * number_of_textures_per_material;
        let mut indecies: Vec<i32> = vec![-1; number_of_indicies];

        self.last_num_textures = num_texs;
        if num_texs == 0 {
            // Set as dummy texture
            renderer.update_or_load_texture(
                &mut self.texture_buffer,
                1,
                1,
                &[&[0u8, 0u8, 0u8, 0u8]],
            );
        } else {
            let mut textures: Vec<Vec<u8>> = vec![];
            let mut texture_data_size = None;
            let mut texture_id_idx_map: HashMap<Id<Texture>, i32> = Default::default();
            for (&material_id, material) in self.materials.iter() {
                let i = (material_id as usize) * number_of_textures_per_material;
                for (j, tex_id) in [
                    material.texture,
                    material.roughness_texture,
                    material.metallic_texture,
                    material.ao_texture,
                    material.normal_texture,
                ]
                .iter()
                .enumerate()
                {
                    if !tex_id.is_null() {
                        let tex_idx = texture_id_idx_map.entry(*tex_id).or_insert_with(|| {
                            let texture = assets.get_mut(*tex_id).expect("Texture should exist");

                            let data = &texture.data;
                            if let Some(texture_data_size) = texture_data_size {
                                // TODO: Should we silently ignore/Print a warning/resize/clip?
                                let (width, height, depth, data_len) = texture_data_size;
                                assert_eq!(width, texture.width);
                                assert_eq!(height, texture.height);
                                assert_eq!(depth, texture.depth);
                                assert_eq!(data_len, data.len());
                            } else {
                                texture_data_size =
                                    Some((texture.width, texture.height, texture.depth, data.len()))
                            }

                            let idx = textures.len();
                            textures.push(data.clone());
                            idx as i32
                        });
                        indecies[i + j] = *tex_idx;
                    }
                }
            }

            if let Some(texture_data_size) = texture_data_size {
                let (width, height, _, _) = texture_data_size;

                let slices: Vec<&[u8]> = textures.iter().map(|tex| tex.as_slice()).collect();

                renderer.update_or_load_texture(
                    &mut self.texture_buffer,
                    width,
                    height,
                    slices.as_slice(),
                );
            }
        }

        renderer.load_buffer(
            &mut self.indicies_buffer,
            bytemuck::cast_slice(indecies.as_slice()),
        );

        result
    }
}
