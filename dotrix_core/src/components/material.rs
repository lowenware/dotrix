use crate::{
    assets::{ Assets, Texture },
    generics::{ Id, Color },
    renderer::{ Renderer, UniformBuffer },
};

/// Material component
#[derive(Default)]
pub struct Material {
    /// Id of a texture asset
    pub texture: Id<Texture>,
    /// Albedo color
    pub albedo: Color,
    /// Pipeline buffer
    pub buffer: UniformBuffer,
}

impl Material {
    /// Loads the [`Material`] into GPU buffers
    pub fn load(&mut self, renderer: &Renderer, assets: &mut Assets) -> bool {
        let mut textures_count = 0;
        if let Some(texture) = assets.get_mut(self.texture) {
            texture.load(renderer);
            textures_count += 1;
        } else {
            return false;
        }

        let uniform = Uniform {
            albedo: self.albedo.into(),
            textures_count
        };

        renderer.load_uniform_buffer(&mut self.buffer, bytemuck::cast_slice(&[uniform]));
        true
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct Uniform {
    albedo: [f32; 4],
    textures_count: u32,
}

unsafe impl bytemuck::Zeroable for Uniform {}
unsafe impl bytemuck::Pod for Uniform {}
