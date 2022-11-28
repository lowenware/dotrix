pub use std::num::NonZeroU32;

/// Texture Wrapper
pub struct Texture {
    pub inner: wgpu::Texture,
}

impl Texture {
    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor) -> TextureView {
        TextureView {
            inner: self.inner.create_view(desc),
        }
    }

    pub fn view<'a, 'b>(&'a self, label: &'b str) -> ViewBuilder<'a, 'b> {
        ViewBuilder {
            texture: self,
            descriptor: wgpu::TextureViewDescriptor {
                label: Some(label),
                ..Default::default()
            },
        }
    }
}

/// Texture builder
pub struct Builder<'a, 'b> {
    pub gpu: &'a crate::Gpu,
    pub descriptor: wgpu::TextureDescriptor<'b>,
}

impl<'a, 'b> Builder<'a, 'b> {
    #[inline(always)]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.descriptor.size.width = width;
        self.descriptor.size.height = height;
        self
    }
    #[inline(always)]
    pub fn layers(mut self, depth_or_array_layers: u32) -> Self {
        self.descriptor.size.depth_or_array_layers = depth_or_array_layers;
        self
    }
    #[inline(always)]
    pub fn mip_level_count(mut self, mip_level_count: u32) -> Self {
        self.descriptor.mip_level_count = mip_level_count;
        self
    }
    #[inline(always)]
    pub fn sample_count(mut self, sample_count: u32) -> Self {
        self.descriptor.sample_count = sample_count;
        self
    }
    #[inline(always)]
    pub fn use_as_render_attachment(mut self) -> Self {
        self.descriptor.usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        self
    }
    #[inline(always)]
    pub fn use_as_storage_binding(mut self) -> Self {
        self.descriptor.usage |= wgpu::TextureUsages::STORAGE_BINDING;
        self
    }
    #[inline(always)]
    pub fn use_as_texture_binding(mut self) -> Self {
        self.descriptor.usage |= wgpu::TextureUsages::TEXTURE_BINDING;
        self
    }
    #[inline(always)]
    pub fn allow_copy_dst(mut self) -> Self {
        self.descriptor.usage |= wgpu::TextureUsages::COPY_DST;
        self
    }
    #[inline(always)]
    pub fn dimension_d1(mut self) -> Self {
        self.descriptor.dimension = wgpu::TextureDimension::D1;
        self
    }
    #[inline(always)]
    pub fn dimension_d2(mut self) -> Self {
        self.descriptor.dimension = wgpu::TextureDimension::D2;
        self
    }
    #[inline(always)]
    pub fn dimension_d3(mut self) -> Self {
        self.descriptor.dimension = wgpu::TextureDimension::D3;
        self
    }
    #[inline(always)]
    pub fn allow_copy_src(mut self) -> Self {
        self.descriptor.usage |= wgpu::TextureUsages::COPY_SRC;
        self
    }
    #[inline(always)]
    pub fn format_depth_f32(mut self) -> Self {
        self.descriptor.format = wgpu::TextureFormat::Depth32Float;
        self
    }
    #[inline(always)]
    pub fn format_rgba_u8(mut self) -> Self {
        self.descriptor.format = wgpu::TextureFormat::Rgba8Uint;
        self
    }
    #[inline(always)]
    pub fn create(self) -> Texture {
        self.gpu.create_texture(&self.descriptor)
    }
}

/// Texture Wrapper
pub struct TextureView {
    pub inner: wgpu::TextureView,
}

pub struct ViewBuilder<'a, 'b> {
    texture: &'a Texture,
    descriptor: wgpu::TextureViewDescriptor<'b>,
}

impl<'a, 'b> ViewBuilder<'a, 'b> {
    #[inline(always)]
    pub fn create(self) -> TextureView {
        self.texture.create_view(&self.descriptor)
    }
    #[inline(always)]
    pub fn format_depth_f32(mut self) -> Self {
        self.descriptor.format = Some(wgpu::TextureFormat::Depth32Float);
        self
    }
    #[inline(always)]
    pub fn format_rgba_u8(mut self) -> Self {
        self.descriptor.format = Some(wgpu::TextureFormat::Rgba8Uint);
        self
    }
    #[inline(always)]
    pub fn dimension_d1(mut self) -> Self {
        self.descriptor.dimension = Some(wgpu::TextureViewDimension::D1);
        self
    }
    #[inline(always)]
    pub fn dimension_d2(mut self) -> Self {
        self.descriptor.dimension = Some(wgpu::TextureViewDimension::D2);
        self
    }
    #[inline(always)]
    pub fn dimension_d3(mut self) -> Self {
        self.descriptor.dimension = Some(wgpu::TextureViewDimension::D3);
        self
    }
    #[inline(always)]
    pub fn aspect_all(mut self) -> Self {
        self.descriptor.aspect = wgpu::TextureAspect::All;
        self
    }
    #[inline(always)]
    pub fn aspect_depth_only(mut self) -> Self {
        self.descriptor.aspect = wgpu::TextureAspect::DepthOnly;
        self
    }
    #[inline(always)]
    pub fn aspect_stencil_only(mut self) -> Self {
        self.descriptor.aspect = wgpu::TextureAspect::StencilOnly;
        self
    }
    #[inline(always)]
    pub fn base_mip_level(mut self, base_mip_level: u32) -> Self {
        self.descriptor.base_mip_level = base_mip_level;
        self
    }
    #[inline(always)]
    pub fn mip_level_count(mut self, mip_level_count: u32) -> Self {
        self.descriptor.mip_level_count = NonZeroU32::new(mip_level_count);
        self
    }
    #[inline(always)]
    pub fn base_array_layer(mut self, base_array_layer: u32) -> Self {
        self.descriptor.base_array_layer = base_array_layer;
        self
    }
    #[inline(always)]
    pub fn array_layer_count(mut self, array_layer_count: u32) -> Self {
        self.descriptor.array_layer_count = NonZeroU32::new(array_layer_count);
        self
    }
}
