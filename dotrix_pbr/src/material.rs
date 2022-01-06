use dotrix_core::assets::Texture;
use dotrix_core::ecs::Mut;
use dotrix_core::renderer::UniformBuffer;
use dotrix_core::{Assets, Color, Id, Renderer};

const DUMMY_TEXTURE: &str = "dotrix::dummy_texture";

/// Material component
#[derive(Default)]
pub struct Material {
    /// Id of a texture asset
    pub texture: Id<Texture>,
    /// Albedo color
    pub albedo: Color,
    /// Roughness (Random scatter)
    pub roughness: f32,
    /// Id of a roughness texture asset
    pub roughness_texture: Id<Texture>,
    /// Metallic (reflectance)
    pub metallic: f32,
    /// Id of a metallic texture asset
    pub metallic_texture: Id<Texture>,
    // Ambient occulsion
    pub ao: f32,
    /// Id of a ao texture asset
    pub ao_texture: Id<Texture>,
    /// Id of a normal map asset
    pub normal_texture: Id<Texture>,
    /// Pipeline buffer
    pub uniform: UniformBuffer,
}

impl Material {
    /// Loads the [`Material`] into GPU buffers
    pub fn load(&mut self, renderer: &Renderer, assets: &mut Assets) -> bool {
        let dummy_id = assets
            .find::<Texture>(DUMMY_TEXTURE)
            .expect("System `dotrix::pbr::material::startup` must be executed");
        if self.texture.is_null() {
            self.texture = dummy_id;
        }
        if self.roughness_texture.is_null() {
            self.roughness_texture = dummy_id;
        }
        if self.metallic_texture.is_null() {
            self.metallic_texture = dummy_id;
        }
        if self.ao_texture.is_null() {
            self.ao_texture = dummy_id;
        }
        if self.normal_texture.is_null() {
            self.normal_texture = dummy_id;
        }

        if let Some(texture) = assets.get_mut(self.texture) {
            texture.load(renderer);
        } else {
            return false;
        }
        if let Some(texture) = assets.get_mut(self.roughness_texture) {
            texture.load(renderer);
        } else {
            return false;
        }
        if let Some(texture) = assets.get_mut(self.metallic_texture) {
            texture.load(renderer);
        } else {
            return false;
        }
        if let Some(texture) = assets.get_mut(self.ao_texture) {
            texture.load(renderer);
        } else {
            return false;
        }
        if let Some(texture) = assets.get_mut(self.normal_texture) {
            texture.load(renderer);
        } else {
            return false;
        }

        let mut has_texture: u32 = 0;
        if self.texture != dummy_id {
            has_texture |= 0b00001;
        }
        if self.roughness_texture != dummy_id {
            has_texture |= 0b00010;
        }
        if self.metallic_texture != dummy_id {
            has_texture |= 0b00100;
        }
        if self.ao_texture != dummy_id {
            has_texture |= 0b01000;
        }
        if self.normal_texture != dummy_id {
            has_texture |= 0b10000;
        }

        let uniform = Uniform {
            albedo: self.albedo.into(),
            has_texture,
            roughness: self.roughness,
            metallic: self.metallic,
            ao: self.ao,
        };

        renderer.load_uniform_buffer(&mut self.uniform, bytemuck::cast_slice(&[uniform]));
        true
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct Uniform {
    albedo: [f32; 4],
    has_texture: u32,
    roughness: f32,
    metallic: f32,
    ao: f32,
}

unsafe impl bytemuck::Zeroable for Uniform {}
unsafe impl bytemuck::Pod for Uniform {}

/// Material startup function
pub fn startup(mut assets: Mut<Assets>) {
    let texture = Texture {
        width: 1,
        height: 1,
        depth: 1,
        data: vec![0, 0, 0, 0],
        ..Default::default()
    };
    assets.store_as(texture, DUMMY_TEXTURE);
}
