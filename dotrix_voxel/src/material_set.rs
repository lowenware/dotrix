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
    material_buffer: Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct MaterialData {
    // Ids of texture in texture array -1 means not precent
    albedo_id: i32,
    roughness_id: i32,
    metallic_id: i32,
    ao_id: i32,
    normal_id: i32,
    padding_a: [i32; 3],
    // These are uses if the texture IDs above are -1
    albedo: [f32; 4],
    roughness: f32,
    metallic: f32,
    ao: f32,
    padding_b: [f32; 1],
}
impl Default for MaterialData {
    fn default() -> Self {
        Self {
            albedo_id: -1,
            roughness_id: -1,
            metallic_id: -1,
            ao_id: -1,
            normal_id: -1,
            albedo: [1., 0.5, 1., 1.],
            roughness: 0.,
            metallic: 0.,
            ao: 0.,
            padding_a: Default::default(),
            padding_b: Default::default(),
        }
    }
}
unsafe impl bytemuck::Zeroable for MaterialData {}
unsafe impl bytemuck::Pod for MaterialData {}

impl Default for MaterialSet {
    fn default() -> Self {
        Self {
            materials: Default::default(),
            last_num_textures: 0,

            texture_buffer: TextureBuffer::new_array("MaterialSet")
                .use_as_texture()
                .rgba_u8norm_srgb(),
            material_buffer: Buffer::uniform("MaterialSetIndicies"),
        }
    }
}

impl MaterialSet {
    /// Set the material for a material ID
    pub fn set_material(&mut self, material_id: u8, material: Material) {
        self.materials.insert(material_id, material);
    }

    /// Clear the material for a material ID
    pub fn clear_material(&mut self, material_id: u8) {
        self.materials.remove(&material_id);
    }

    /// The material buffer. Contains the materials data for the gpu
    /// Must be loaed before use with `[load]`
    pub fn get_material_buffer(&self) -> &Buffer {
        &self.material_buffer
    }

    /// The texture array buffer. Contains the textures as referenced by the material buffer for the gpu
    /// Must be loaed before use with `[load]`
    pub fn get_texture_buffer(&self) -> &TextureBuffer {
        &self.texture_buffer
    }

    /// Returns true if a full rebind is required
    /// returns false if rebind is not required, but may still update textures
    /// by replacing current textures.
    pub fn load(&mut self, renderer: &Renderer, assets: &Assets) -> bool {
        let mut result = false;

        let number_of_materials = 256;
        let number_of_textures_per_material = 5;
        let mut material_data: Vec<MaterialData> = vec![Default::default(); number_of_materials];

        let mut num_texs_found = 0;
        let mut textures: Vec<Vec<u8>> = vec![];
        let mut texture_data_size = None;
        let mut texture_id_idx_map: HashMap<Id<Texture>, i32> = Default::default();
        for (&material_id, material) in self.materials.iter() {
            let i = material_id as usize;
            let mut tex_ids = vec![-1; number_of_textures_per_material];
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
                    if let Some(texture) = assets.get(*tex_id) {
                        let tex_idx = texture_id_idx_map.entry(*tex_id).or_insert_with(|| {
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
                            num_texs_found += 1;
                            let idx = textures.len();
                            textures.push(data.clone());
                            idx as i32
                        });
                        tex_ids[j] = *tex_idx;
                    }
                }
                material_data[i].albedo_id = tex_ids[0];
                material_data[i].roughness_id = tex_ids[1];
                material_data[i].metallic_id = tex_ids[2];
                material_data[i].ao_id = tex_ids[3];
                material_data[i].normal_id = tex_ids[4];
                material_data[i].albedo = material.albedo.into();
                material_data[i].roughness = material.roughness;
                material_data[i].metallic = material.metallic;
                material_data[i].ao = material.ao;
            }

            if num_texs_found != self.last_num_textures {
                // Full reload and bind required
                // because number of textures was changed
                result = true;
            }

            if result {
                self.texture_buffer.unload();
            }

            if num_texs_found == 0 {
                // Set as dummy texture
                renderer.update_or_load_texture(
                    &mut self.texture_buffer,
                    1,
                    1,
                    &[&[0u8, 0u8, 0u8, 0u8]],
                );
            } else if let Some(texture_data_size) = texture_data_size {
                let (width, height, _, _) = texture_data_size;

                let slices: Vec<&[u8]> = textures.iter().map(|tex| tex.as_slice()).collect();

                renderer.update_or_load_texture(
                    &mut self.texture_buffer,
                    width,
                    height,
                    slices.as_slice(),
                );
            } else {
                unreachable!();
            }

            self.last_num_textures = num_texs_found;
        }

        renderer.load_buffer(
            &mut self.material_buffer,
            bytemuck::cast_slice(material_data.as_slice()),
        );

        result
    }
}
