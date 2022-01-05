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

        let albedo = MaterialUniform {
            color: self.albedo.into(),
            has_texture: !(self.texture == dummy_id) as u32,
            ..Default::default()
        };
        let roughness = MaterialUniform {
            color: [
                self.roughness,
                self.roughness,
                self.roughness,
                self.roughness,
            ],
            has_texture: !(self.roughness_texture == dummy_id) as u32,
            ..Default::default()
        };
        let metallic = MaterialUniform {
            color: [self.metallic, self.metallic, self.metallic, self.metallic],
            has_texture: !(self.metallic_texture == dummy_id) as u32,
            ..Default::default()
        };
        let ao = MaterialUniform {
            color: [self.ao, self.ao, self.ao, self.ao],
            has_texture: !(self.ao_texture == dummy_id) as u32,
            ..Default::default()
        };

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

        let uniform = Uniform {
            albedo,
            roughness,
            metallic,
            ao,
        };

        renderer.load_uniform_buffer(&mut self.uniform, bytemuck::cast_slice(&[uniform]));
        true
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct MaterialUniform {
    color: [f32; 4],
    has_texture: u32,
    reserved: [f32; 3],
}
unsafe impl bytemuck::Zeroable for MaterialUniform {}
unsafe impl bytemuck::Pod for MaterialUniform {}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct Uniform {
    albedo: MaterialUniform,
    roughness: MaterialUniform,
    metallic: MaterialUniform,
    ao: MaterialUniform,
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
