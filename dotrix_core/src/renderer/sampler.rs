use super::Context;

/// Texture Sampler
#[derive(Default)]
pub struct Sampler {
    /// WGPU sampler instance
    pub wgpu_sampler: Option<wgpu::Sampler>,
}

impl Sampler {
    /// Loads the Sampler
    pub(crate) fn load(&mut self, ctx: &Context) {
        if self.wgpu_sampler.is_some() {
            return;
        }
        self.wgpu_sampler = Some(ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));
    }

    /// Checks if the Sampler is empty
    pub fn loaded(&self) -> bool {
        self.wgpu_sampler.is_some()
    }

    /// Release all resources used by the Sampler
    pub fn unload(&mut self) {
        self.wgpu_sampler.take();
    }

    /// Unwraps WGPU Sampler
    pub fn get(&self) -> &wgpu::Sampler {
        self.wgpu_sampler.as_ref().expect("Sampler must be loaded")
    }
}
