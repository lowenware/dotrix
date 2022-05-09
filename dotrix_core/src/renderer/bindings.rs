use super::{Access, Context, GpuBuffer, GpuTexture, PipelineInstance, Sampler};

/// Rendering stage
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    /// Vertex shader stage
    Vertex,
    /// Fragment shader stage
    Fragment,
    /// Compute shader stage
    Compute,
    /// Any stage
    All,
}

impl From<&Stage> for wgpu::ShaderStages {
    fn from(obj: &Stage) -> Self {
        match obj {
            Stage::All => wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            Stage::Vertex => wgpu::ShaderStages::VERTEX,
            Stage::Fragment => wgpu::ShaderStages::FRAGMENT,
            Stage::Compute => wgpu::ShaderStages::COMPUTE,
        }
    }
}

/// Binding types (Label, Stage, Buffer)
pub enum Binding<'a> {
    /// Uniform binding
    Uniform(&'a str, Stage, Box<&'a dyn GpuBuffer>),
    /// Texture binding
    Texture(&'a str, Stage, Box<&'a dyn GpuTexture>),
    /// Cube Texture binding
    TextureCube(&'a str, Stage, Box<&'a dyn GpuTexture>),
    /// 2D Texture Array binding
    TextureArray(&'a str, Stage, Box<&'a dyn GpuTexture>),
    /// 3D Texture binding
    Texture3D(&'a str, Stage, Box<&'a dyn GpuTexture>),
    /// Storage texture binding
    StorageTexture(&'a str, Stage, Box<&'a dyn GpuTexture>, Access),
    /// Storage texture cube binding
    StorageTextureCube(&'a str, Stage, Box<&'a dyn GpuTexture>, Access),
    /// Storage 2D texture array binding
    StorageTextureArray(&'a str, Stage, Box<&'a dyn GpuTexture>, Access),
    /// Storage texture binding 3D
    StorageTexture3D(&'a str, Stage, Box<&'a dyn GpuTexture>, Access),
    /// Texture sampler binding
    Sampler(&'a str, Stage, &'a Sampler),
    /// Storage binding
    Storage(&'a str, Stage, Box<&'a dyn GpuBuffer>),
}

impl<'a> Binding<'a> {
    /// Uniform binding
    pub fn uniform<Buffer: GpuBuffer>(
        label: &'a str,
        stage: Stage,
        asset: &'a Buffer,
    ) -> Binding<'a> {
        Self::Uniform(label, stage, Box::new(asset))
    }
    /// Texture binding
    pub fn texture<Texture: GpuTexture>(
        label: &'a str,
        stage: Stage,
        asset: &'a Texture,
    ) -> Binding<'a> {
        Self::Texture(label, stage, Box::new(asset))
    }
    /// Cube Texture binding
    pub fn texture_cube<Texture: GpuTexture>(
        label: &'a str,
        stage: Stage,
        asset: &'a Texture,
    ) -> Binding<'a> {
        Self::TextureCube(label, stage, Box::new(asset))
    }
    /// 2D Texture Array binding
    pub fn texture_array<Texture: GpuTexture>(
        label: &'a str,
        stage: Stage,
        asset: &'a Texture,
    ) -> Binding<'a> {
        Self::TextureArray(label, stage, Box::new(asset))
    }
    /// 3D Texture binding
    pub fn texture_3d<Texture: GpuTexture>(
        label: &'a str,
        stage: Stage,
        asset: &'a Texture,
    ) -> Binding<'a> {
        Self::Texture3D(label, stage, Box::new(asset))
    }
    /// Storage texture binding
    pub fn storage_texture<Texture: GpuTexture>(
        label: &'a str,
        stage: Stage,
        asset: &'a Texture,
        access: Access,
    ) -> Binding<'a> {
        Self::StorageTexture(label, stage, Box::new(asset), access)
    }
    /// Storage texture cube binding
    pub fn storage_texture_cube<Texture: GpuTexture>(
        label: &'a str,
        stage: Stage,
        asset: &'a Texture,
        access: Access,
    ) -> Binding<'a> {
        Self::StorageTextureCube(label, stage, Box::new(asset), access)
    }
    /// Storage 2D texture array binding
    pub fn storage_texture_array<Texture: GpuTexture>(
        label: &'a str,
        stage: Stage,
        asset: &'a Texture,
        access: Access,
    ) -> Binding<'a> {
        Self::StorageTextureArray(label, stage, Box::new(asset), access)
    }
    /// Storage texture binding 3D
    pub fn storage_texture_3d<Texture: GpuTexture>(
        label: &'a str,
        stage: Stage,
        asset: &'a Texture,
        access: Access,
    ) -> Binding<'a> {
        Self::StorageTexture3D(label, stage, Box::new(asset), access)
    }
    /// Texture sampler binding
    pub fn sampler(label: &'a str, stage: Stage, asset: &'a Sampler) -> Binding<'a> {
        Self::Sampler(label, stage, asset)
    }
    /// Storage binding
    pub fn storage<Buffer: GpuBuffer>(
        label: &'a str,
        stage: Stage,
        asset: &'a Buffer,
    ) -> Binding<'a> {
        Self::Storage(label, stage, Box::new(asset))
    }
}

/// Bind Group holding bindings
pub struct BindGroup<'a> {
    /// Text label of the Bind group
    pub label: &'a str,
    /// List of bindings
    pub bindings: Vec<Binding<'a>>,
}

impl<'a> BindGroup<'a> {
    /// Constructs new Bind Group
    pub fn new(label: &'a str, bindings: Vec<Binding<'a>>) -> Self {
        Self { label, bindings }
    }

    /// Constructs WGPU BindGroupLayout for the `BindGroup`
    pub fn layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let entries = self
            .bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| match binding {
                Binding::Uniform(_, stage, _) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                Binding::Texture(_, stage, texture) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: texture.get_texture().sample_type(),
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                Binding::TextureCube(_, stage, texture) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: texture.get_texture().sample_type(),
                        view_dimension: wgpu::TextureViewDimension::Cube,
                    },
                    count: None,
                },
                Binding::TextureArray(_, stage, texture) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: texture.get_texture().sample_type(),
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                Binding::Texture3D(_, stage, texture) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: texture.get_texture().sample_type(),
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
                Binding::StorageTexture(_, stage, texture, access) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::StorageTexture {
                        access: access.into(),
                        format: texture.get_texture().format,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                Binding::StorageTextureCube(_, stage, texture, access) => {
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: stage.into(),
                        ty: wgpu::BindingType::StorageTexture {
                            access: access.into(),
                            format: texture.get_texture().format,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                        },
                        count: None,
                    }
                }
                Binding::StorageTextureArray(_, stage, texture, access) => {
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: stage.into(),
                        ty: wgpu::BindingType::StorageTexture {
                            access: access.into(),
                            format: texture.get_texture().format,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    }
                }
                Binding::StorageTexture3D(_, stage, texture, access) => {
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: stage.into(),
                        ty: wgpu::BindingType::StorageTexture {
                            access: access.into(),
                            format: texture.get_texture().format,
                            view_dimension: wgpu::TextureViewDimension::D3,
                        },
                        count: None,
                    }
                }
                Binding::Sampler(_, stage, _) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                Binding::Storage(_, stage, storage) => {
                    let read_only = !storage.get_buffer().can_write();
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: stage.into(),
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                }
            })
            .collect::<Vec<_>>();

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(self.label),
            entries: entries.as_slice(),
        })
    }
}

/// Pipeline Bindings
#[derive(Default)]
pub struct Bindings {
    /// List of `wgpu::BindGroup`
    pub wgpu_bind_groups: Vec<wgpu::BindGroup>,
}

impl Bindings {
    pub(crate) fn load<'a>(
        &mut self,
        ctx: &Context,
        pipeline_instance: &PipelineInstance,
        bind_groups: &[BindGroup<'a>],
    ) {
        let wgpu_bind_groups_layout = match pipeline_instance {
            PipelineInstance::Render(render) => &render.wgpu_bind_groups_layout,
            PipelineInstance::Compute(compute) => &compute.wgpu_bind_groups_layout,
        };
        self.wgpu_bind_groups = wgpu_bind_groups_layout
            .iter()
            .enumerate()
            .map(|(group, wgpu_bind_group_layout)| {
                ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: wgpu_bind_group_layout,
                    entries: bind_groups[group]
                        .bindings
                        .iter()
                        .enumerate()
                        .map(|(binding, entry)| wgpu::BindGroupEntry {
                            binding: binding as u32,
                            resource: match entry {
                                Binding::Uniform(_, _, uniform) => {
                                    uniform.get_buffer().get().as_entire_binding()
                                }
                                Binding::Texture(_, _, texture)
                                | Binding::TextureCube(_, _, texture)
                                | Binding::TextureArray(_, _, texture)
                                | Binding::Texture3D(_, _, texture)
                                | Binding::StorageTexture(_, _, texture, _)
                                | Binding::StorageTextureCube(_, _, texture, _)
                                | Binding::StorageTextureArray(_, _, texture, _)
                                | Binding::StorageTexture3D(_, _, texture, _) => {
                                    wgpu::BindingResource::TextureView(texture.get_texture().get())
                                }
                                Binding::Sampler(_, _, sampler) => {
                                    wgpu::BindingResource::Sampler(sampler.get())
                                }
                                Binding::Storage(_, _, storage) => {
                                    storage.get_buffer().get().as_entire_binding()
                                }
                            },
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    label: None,
                })
            })
            .collect::<Vec<_>>();
    }

    /// Returns true if bindings was loaded to GPU
    pub fn loaded(&self) -> bool {
        !self.wgpu_bind_groups.is_empty()
    }

    /// Unloads bindings from GPU
    pub fn unload(&mut self) {
        self.wgpu_bind_groups.clear();
    }
}
