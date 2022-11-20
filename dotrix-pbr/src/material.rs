use dotrix_assets as assets;
use dotrix_image as image;
use dotrix_types::{id, Color, Id};

pub const MAP_DISABLED: u32 = 0xFFFFFFFF;

/// Material component
pub struct Material {
    /// Label or material name
    pub label: String,
    /// Albedo color
    pub albedo: Color,
    /// Id of a texture asset
    pub albedo_map: Id<image::Image>,
    // Ambient occulsion
    pub ambient_occlusion: f32,
    /// Id of a ao texture asset
    pub ambient_occlusion_map: Id<image::Image>,
    /// Metallic (reflectance)
    pub metallic: f32,
    /// Id of a metallic texture asset
    pub metallic_map: Id<image::Image>,
    /// Id of a normal map asset
    pub normal_map: Id<image::Image>,
    /// Roughness (Random scatter)
    pub roughness: f32,
    /// Id of a roughness texture asset
    pub roughness_map: Id<image::Image>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            label: String::from("dotrix::material"),
            albedo: Color::white(),
            albedo_map: Id::default(),
            ambient_occlusion: 1.0,
            ambient_occlusion_map: Id::default(),
            metallic: 1.0,
            metallic_map: Id::default(),
            normal_map: Id::default(),
            roughness: 1.0,
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
    /// Index of Color map in the buffer + 3 reserved values
    pub maps_2: [u32; 4],
}

unsafe impl bytemuck::Pod for MaterialUniform {}
unsafe impl bytemuck::Zeroable for MaterialUniform {}

impl id::NameSpace for Material {
    fn namespace() -> u64 {
        assets::NAMESPACE | 0x21
    }
}

impl assets::Asset for Material {
    fn name(&self) -> &str {
        &self.label
    }

    fn namespace(&self) -> u64 {
        <Self as id::NameSpace>::namespace()
    }
}
