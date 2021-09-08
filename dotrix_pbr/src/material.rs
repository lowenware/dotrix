use dotrix_core::{ Assets, Id, Color, Renderer };
use dotrix_core::assets::Texture;
use dotrix_core::ecs::Mut;
use dotrix_core::renderer::UniformBuffer;

const DUMMY_TEXTURE: &str = "dotrix::dummy_texture";

/// Material component
#[derive(Default)]
pub struct Material {
    /// Id of a texture asset
    pub texture: Id<Texture>,
    /// Albedo color
    pub albedo: Color,
    /// Pipeline buffer
    pub uniform: UniformBuffer,
    /// Has texture
    pub dummy_texture: bool,
}

impl Material {
    /// Loads the [`Material`] into GPU buffers
    pub fn load(&mut self, renderer: &Renderer, assets: &mut Assets) -> bool {
        if self.texture.is_null() {
            self.dummy_texture = true;
            self.texture = assets.find(DUMMY_TEXTURE)
                .expect("System `dotrix::pbr::material::startup` must be executed");
        }

        if let Some(texture) = assets.get_mut(self.texture) {
            texture.load(renderer);
        } else {
            return false;
        }

        let uniform = Uniform {
            albedo: self.albedo.into(),
            has_texture: !self.dummy_texture as u32,
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
