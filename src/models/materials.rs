use super::{Color, Image};
use crate::loaders::Asset;
use crate::utils::Id;

/// Material component
#[derive(Debug)]
pub struct Material {
    /// Label or material name
    pub name: String,
    /// Albedo color
    pub albedo: Color<f32>,
    /// Id of a texture asset
    pub albedo_map: Id<Image>,
    // Ambient occulsion
    pub occlusion_factor: f32,
    /// Id of a ao texture asset
    pub occlusion_map: Id<Image>,
    /// Metallic (reflectance)
    pub metallic_factor: f32,
    /// Id of a metallic texture asset
    pub metallic_map: Id<Image>,
    /// Id of a normal map asset
    pub normal_map: Id<Image>,
    /// Roughness (Random scatter)
    pub roughness_factor: f32,
    /// Id of a roughness texture asset
    pub roughness_map: Id<Image>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: String::from("dotrix::material"),
            albedo: Color::white(),
            albedo_map: Id::default(),
            occlusion_factor: 1.0,
            occlusion_map: Id::default(),
            metallic_factor: 1.0,
            metallic_map: Id::default(),
            normal_map: Id::default(),
            roughness_factor: 1.0,
            roughness_map: Id::default(),
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct MaterialUniform {
    /// Albedo color RGBA
    pub color: [f32; 4],
    /// Order: ambient_occlusion, metallic, roughness
    pub options: [f32; 4],
    /// Indices of PBR maps in the buffer
    /// Order: ambient_occlusion, metallic, normal, roughness
    pub maps_1: [u32; 4],
    /// Index of Color map in the buffer + 3 reserved value¨
    pub maps_2: [u32; 4],
}

impl From<&Material> for MaterialUniform {
    fn from(value: &Material) -> Self {
        Self {
            color: (&value.albedo).into(),
            options: [
                value.occlusion_factor,
                value.metallic_factor,
                value.roughness_factor,
                0.0,
            ],
            maps_1: [0, 0, 0, 0],
            maps_2: [0, 0, 0, 0],
        }
    }
}

unsafe impl bytemuck::Pod for MaterialUniform {}
unsafe impl bytemuck::Zeroable for MaterialUniform {}

impl Asset for Material {
    fn name(&self) -> &str {
        &self.name
    }
}
