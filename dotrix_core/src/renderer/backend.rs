/// WGPU backend wrapper module
use std::{borrow::Cow, collections::HashMap};
use wgpu;
use wgpu::util::DeviceExt;
use winit;

use crate::{assets::Shader, color::Color, id::Id};

use super::{AttributeFormat, BindGroup, Binding, DepthBufferMode, Options, PipelineLayout, Stage};

pub(crate) struct Context {
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    sur_desc: wgpu::SurfaceConfiguration,
    depth_buffer: wgpu::TextureView,
    frame: Option<wgpu::SurfaceTexture>,
    encoder: Option<wgpu::CommandEncoder>,
    pipelines: HashMap<Id<Shader>, PipelineBackend>,
}

impl Context {
    pub(crate) fn bind_frame(&mut self, clear_color: &Color) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.sur_desc);
                self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture")
            }
        };

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor { label: None };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);
        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.r as f64,
                            g: clear_color.g as f64,
                            b: clear_color.b as f64,
                            a: clear_color.a as f64,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        }
        self.encoder = Some(encoder);
        self.frame = Some(frame);
    }

    pub(crate) fn release_frame(&mut self) {
        if let Some(encoder) = self.encoder.take() {
            self.queue.submit(Some(encoder.finish()));
        }
        if let Some(frame) = self.frame.take() {
            frame.present();
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.sur_desc.width = width;
            self.sur_desc.height = height;

            self.surface.configure(&self.device, &self.sur_desc);
            self.depth_buffer = create_depth_buffer(&self.device, width, height);
        }
    }

    pub(crate) fn drop_pipeline(&mut self, shader: Id<Shader>) {
        self.pipelines.remove(&shader);
    }

    pub(crate) fn drop_all_pipelines(&mut self) {
        self.pipelines.clear();
    }

    pub(crate) fn add_pipeline(&mut self, shader: Id<Shader>, pipeline_backend: PipelineBackend) {
        self.pipelines.insert(shader, pipeline_backend);
    }

    pub(crate) fn has_pipeline(&self, shader: Id<Shader>) -> bool {
        self.pipelines.contains_key(&shader)
    }

    pub(crate) fn pipeline(&self, shader: Id<Shader>) -> Option<&PipelineBackend> {
        self.pipelines.get(&shader)
    }

    pub(crate) fn run_render_pipeline(
        &mut self,
        shader: Id<Shader>,
        vertex_buffer: &VertexBuffer,
        bindings: &Bindings,
        options: &Options,
    ) {
        if let Some(pipeline) = self.pipelines.get(&shader) {
            let pipeline_backend = pipeline.instance.render();
            let depth_buffer_mode = pipeline_backend.depth_buffer_mode;
            let encoder = self.encoder.as_mut().expect("WGPU encoder must be set");

            let frame = self.frame.as_ref().expect("WGPU frame must be set");
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: if depth_buffer_mode != DepthBufferMode::Disabled {
                    Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_buffer,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }),
                        stencil_ops: None,
                    })
                } else {
                    None
                },
            });

            rpass.push_debug_group("Prepare to run pipeline");
            rpass.set_pipeline(&pipeline_backend.wgpu_pipeline);

            if let Some(scissors_rect) = options.scissors_rect.as_ref() {
                rpass.set_scissor_rect(
                    scissors_rect.clip_min_x,
                    scissors_rect.clip_min_y,
                    scissors_rect.width,
                    scissors_rect.height,
                );
            }

            for (index, wgpu_bind_group) in bindings.wgpu_bind_groups.iter().enumerate() {
                rpass.set_bind_group(index as u32, wgpu_bind_group, &[]);
            }
            rpass.set_vertex_buffer(0, vertex_buffer.get().slice(..));
            rpass.pop_debug_group();

            let count = vertex_buffer.count;

            if let Some(indices_buffer) = vertex_buffer.indices().as_ref() {
                rpass.insert_debug_marker("Draw indexed");
                rpass.set_index_buffer(indices_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..count, 0, options.start_index..options.end_index);
            } else {
                rpass.insert_debug_marker("Draw");
                rpass.draw(0..count, options.start_index..options.end_index);
            }
        }
    }

    pub(crate) fn run_compute_pipeline(
        &mut self,
        shader: Id<Shader>,
        bindings: &Bindings,
        work_groups: &WorkGroups,
    ) {
        if let Some(pipeline) = self.pipelines.get(&shader) {
            let pipeline_backend = pipeline.instance.compute();
            let encoder = self.encoder.as_mut().expect("WGPU encoder must be set");

            // compute pass
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&pipeline_backend.wgpu_pipeline);
            for (index, wgpu_bind_group) in bindings.wgpu_bind_groups.iter().enumerate() {
                cpass.set_bind_group(index as u32, wgpu_bind_group, &[]);
            }
            cpass.dispatch(work_groups.x, work_groups.y, work_groups.z);
        }
    }
}

pub(crate) async fn init(window: &winit::window::Window) -> Context {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find an appropiate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
                limits: wgpu::Limits::default(),
            },
            None, // Some(&std::path::Path::new("./wgpu-trace/")),
        )
        .await
        .expect("Failed to create device");

    let size = window.inner_size();

    let sur_desc = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    surface.configure(&device, &sur_desc);
    let depth_buffer = create_depth_buffer(&device, size.width, size.height);

    Context {
        adapter,
        device,
        queue,
        surface,
        sur_desc,
        depth_buffer,
        frame: None,
        encoder: None,
        pipelines: std::collections::HashMap::new(),
    }
}

fn create_depth_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let buffer_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = wgpu::TextureDescriptor {
        label: Some("Depth Buffer"),
        size: buffer_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
    };

    device
        .create_texture(&texture)
        .create_view(&wgpu::TextureViewDescriptor::default())
}

/// Buffer for vertices attributes
#[derive(Default)]
pub struct VertexBuffer {
    /// Packed vertex attributes
    attributes: Option<wgpu::Buffer>,
    /// Optional Indices buffer
    indices: Option<wgpu::Buffer>,
    count: u32,
}

impl VertexBuffer {
    /// Loads data into the vertex buffer
    pub(crate) fn load<'a>(
        &mut self,
        ctx: &Context,
        attributes: &'a [u8],
        indices: Option<&'a [u8]>,
        count: u32,
    ) {
        if let Some(buffer) = self.attributes.as_ref() {
            ctx.queue.write_buffer(buffer, 0, attributes);
        } else {
            self.attributes = Some(ctx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("VertexBuffer"),
                    contents: attributes,
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));
        }

        if let Some(buffer) = self.indices.as_ref() {
            let indices = indices.expect("Indexed meshed can't be reloaded without indices");
            ctx.queue.write_buffer(buffer, 0, indices);
        } else {
            self.indices = indices.map(|contents| {
                ctx.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("IndexBuffer"),
                        contents,
                        usage: wgpu::BufferUsages::INDEX,
                    })
            });
        }

        self.count = count;
    }

    /// Checks if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.attributes.is_none()
    }

    /// Release all resources used by the buffer
    pub fn empty(&mut self) {
        self.attributes.take();
        self.indices.take();
    }

    fn get(&self) -> &wgpu::Buffer {
        self.attributes
            .as_ref()
            .expect("Attributes buffer must be loaded")
    }

    fn indices(&self) -> Option<&wgpu::Buffer> {
        self.indices.as_ref()
    }
}

/// Texture Buffer
#[derive(Default)]
pub struct TextureBuffer {
    wgpu_texture_view: Option<wgpu::TextureView>,
}

impl TextureBuffer {
    /// Loads data into the texture buffer
    pub(crate) fn load<'a>(&mut self, ctx: &Context, width: u32, height: u32, layers: &[&'a [u8]]) {
        let depth_or_array_layers = layers.len() as u32;

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };

        let layer_size = wgpu::Extent3d {
            depth_or_array_layers: 1,
            ..size
        };

        let max_mips = 1; //layer_size.max_mips();

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TextureBuffer"),
            size,
            mip_level_count: max_mips as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        self.wgpu_texture_view = Some(texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            dimension: Some(if depth_or_array_layers == 6 {
                wgpu::TextureViewDimension::Cube
            } else {
                wgpu::TextureViewDimension::D2
            }),
            ..wgpu::TextureViewDescriptor::default()
        }));

        for (i, data) in layers.iter().enumerate() {
            let bytes_per_row = std::num::NonZeroU32::new(data.len() as u32 / height).unwrap();

            ctx.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(std::num::NonZeroU32::new(height).unwrap()),
                },
                layer_size,
            );
        }
    }

    /// Checks if buffer is empty
    pub fn loaded(&self) -> bool {
        self.wgpu_texture_view.is_some()
    }

    /// Release all resources used by the buffer
    pub fn unload(&mut self) {
        self.wgpu_texture_view.take();
    }

    fn get(&self) -> &wgpu::TextureView {
        self.wgpu_texture_view
            .as_ref()
            .expect("Texture must be loaded")
    }
}

/// Uniform Buffer
#[derive(Default)]
pub struct UniformBuffer {
    wgpu_buffer: Option<wgpu::Buffer>,
}

impl UniformBuffer {
    /// Loads data into the uniform buffer
    pub(crate) fn load<'a>(&mut self, ctx: &Context, data: &'a [u8]) {
        if let Some(buffer) = self.wgpu_buffer.as_ref() {
            ctx.queue.write_buffer(buffer, 0, data);
        } else {
            self.wgpu_buffer = Some(ctx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("UniformBuffer"),
                    contents: data,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                },
            ));
        }
    }

    /// Checks if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.wgpu_buffer.is_none()
    }

    /// Release all resources used by the buffer
    pub fn empty(&mut self) {
        self.wgpu_buffer.take();
    }

    fn get(&self) -> &wgpu::Buffer {
        self.wgpu_buffer
            .as_ref()
            .expect("Uniform buffer must be loaded")
    }
}

/// Texture Sampler
#[derive(Default)]
pub struct Sampler {
    wgpu_sampler: Option<wgpu::Sampler>,
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
    pub fn is_empty(&self) -> bool {
        self.wgpu_sampler.is_none()
    }

    /// Release all resources used by the Sampler
    pub fn empty(&mut self) {
        self.wgpu_sampler.take();
    }

    fn get(&self) -> &wgpu::Sampler {
        self.wgpu_sampler.as_ref().expect("Sampler must be loaded")
    }
}

enum StorageBufferMode {
    Read,
    ReadWrite,
}

impl Default for StorageBufferMode {
    fn default() -> Self {
        Self::Read
    }
}

/// Storage Buffer
#[derive(Default)]
pub struct StorageBuffer {
    mode: StorageBufferMode,
    wgpu_buffer: Option<wgpu::Buffer>,
}

impl StorageBuffer {
    /// Create a read only storage buffer
    pub fn new_readonly() -> Self {
        Self {
            mode: StorageBufferMode::Read,
            wgpu_buffer: Default::default(),
        }
    }

    /// Create a read-write storage buffer
    pub fn new_readwrite() -> Self {
        Self {
            mode: StorageBufferMode::ReadWrite,
            wgpu_buffer: Default::default(),
        }
    }

    /// Loads data into the storage buffer
    pub(crate) fn load<'a>(&mut self, ctx: &Context, data: &'a [u8]) {
        if let Some(buffer) = self.wgpu_buffer.as_ref() {
            ctx.queue.write_buffer(buffer, 0, data);
        } else {
            let usage = match self.mode {
                StorageBufferMode::Read => {
                    wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::UNIFORM
                        | wgpu::BufferUsages::COPY_DST
                }
                StorageBufferMode::ReadWrite => {
                    wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::UNIFORM
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC
                }
            };
            self.wgpu_buffer = Some(ctx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("StorageBuffer"),
                    contents: data,
                    usage,
                },
            ));
        }
    }

    /// Checks if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.wgpu_buffer.is_none()
    }

    /// Release all resources used by the buffer
    pub fn empty(&mut self) {
        self.wgpu_buffer.take();
    }

    fn get(&self) -> &wgpu::Buffer {
        self.wgpu_buffer
            .as_ref()
            .expect("Storage buffer must be loaded")
    }
}

/// Render pipeline backend
pub struct RenderPipelineBackend {
    /// WGPU pipeline
    wgpu_pipeline: wgpu::RenderPipeline,
    depth_buffer_mode: DepthBufferMode,
}

/// Compute pipeline backend
pub struct ComputePipelineBackend {
    /// WGPU pipeline
    wgpu_pipeline: wgpu::ComputePipeline,
}

/// Pipeline backend
pub enum PipelineInstance {
    Render(RenderPipelineBackend),
    Compute(ComputePipelineBackend),
}

impl PipelineInstance {
    fn render(&self) -> &RenderPipelineBackend {
        match self {
            Self::Render(pipeline) => pipeline,
            Self::Compute(_) => panic!("Compute pipeline used for rendering"),
        }
    }
    fn compute(&self) -> &ComputePipelineBackend {
        match self {
            Self::Compute(pipeline) => pipeline,
            Self::Render(_) => panic!("Render pipeline used for rendering"),
        }
    }
}

/// Numbers of Work Groups in all directions
pub struct WorkGroups {
    /// Number of Work Groups in X direction
    pub x: u32,
    /// Number of Work Groups in Y direction
    pub y: u32,
    /// Number of Work Groups in Z direction
    pub z: u32,
}

/// Compute pipeline backend
pub struct PipelineBackend {
    /// WGPU bind group layout
    wgpu_bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    /// WGPU pipeline
    instance: PipelineInstance,
}

#[inline(always)]
fn visibility(stage: &Stage) -> wgpu::ShaderStages {
    match stage {
        Stage::All => wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        Stage::Vertex => wgpu::ShaderStages::VERTEX,
        Stage::Fragment => wgpu::ShaderStages::FRAGMENT,
        Stage::Compute => wgpu::ShaderStages::COMPUTE,
    }
}

impl PipelineBackend {
    pub(crate) fn new(ctx: &Context, pipeline: &PipelineLayout) -> Self {
        let wgpu_shader_module = pipeline.shader.module.get();
        let wgpu_bind_group_layouts = pipeline
            .bindings
            .iter()
            .map(|bind_group_layout| Self::bind_group_layout(&ctx.device, bind_group_layout))
            .collect::<Vec<_>>();

        // create pipeline layout
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: wgpu_bind_group_layouts
                    .iter()
                    .collect::<Vec<_>>()
                    .as_slice(),
                push_constant_ranges: &[],
            });

        let instance = if let Some(mesh) = pipeline.mesh {
            let depth_buffer_mode = pipeline.options.depth_buffer_mode;

            // render pipeline: prepare vertex buffers layout
            let mut vertex_array_stride = 0;
            let vertex_attributes = mesh
                .vertex_buffer_layout()
                .iter()
                .enumerate()
                .map(|(index, attr)| {
                    let offset = vertex_array_stride;
                    vertex_array_stride += attr.size();
                    wgpu::VertexAttribute {
                        format: match attr {
                            AttributeFormat::Float32 => wgpu::VertexFormat::Float32,
                            AttributeFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
                            AttributeFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
                            AttributeFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
                            AttributeFormat::Uint16x2 => wgpu::VertexFormat::Uint16x2,
                            AttributeFormat::Uint16x4 => wgpu::VertexFormat::Uint16x4,
                            AttributeFormat::Uint32 => wgpu::VertexFormat::Uint32,
                            AttributeFormat::Uint32x2 => wgpu::VertexFormat::Uint32x2,
                            AttributeFormat::Uint32x3 => wgpu::VertexFormat::Uint32x3,
                            AttributeFormat::Uint32x4 => wgpu::VertexFormat::Uint32x4,
                        },
                        offset: offset as u64,
                        shader_location: index as u32,
                    }
                })
                .collect::<Vec<_>>();

            let vertex_buffers = [wgpu::VertexBufferLayout {
                array_stride: vertex_array_stride as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: vertex_attributes.as_slice(),
            }];

            // create the pipeline
            let wgpu_pipeline =
                ctx.device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(&pipeline.label),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: wgpu_shader_module,
                            entry_point: "vs_main",
                            buffers: &vertex_buffers,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: wgpu_shader_module,
                            entry_point: "fs_main",
                            targets: &[if depth_buffer_mode == DepthBufferMode::Disabled {
                                wgpu::ColorTargetState {
                                    format: ctx.sur_desc.format,
                                    blend: Some(wgpu::BlendState {
                                        color: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::One,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                        alpha: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            dst_factor: wgpu::BlendFactor::One,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                    }),
                                    write_mask: wgpu::ColorWrites::ALL,
                                }
                            } else {
                                wgpu::ColorTargetState {
                                    format: ctx.sur_desc.format,
                                    blend: Some(wgpu::BlendState {
                                        color: wgpu::BlendComponent::REPLACE,
                                        alpha: wgpu::BlendComponent::REPLACE,
                                    }),
                                    write_mask: wgpu::ColorWrites::ALL,
                                }
                            }],
                        }),
                        primitive: wgpu::PrimitiveState {
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: if !pipeline.options.disable_cull_mode {
                                Some(wgpu::Face::Back)
                            } else {
                                None
                            },
                            ..Default::default()
                        },
                        depth_stencil: if depth_buffer_mode != DepthBufferMode::Disabled {
                            Some(wgpu::DepthStencilState {
                                format: wgpu::TextureFormat::Depth32Float,
                                depth_write_enabled: depth_buffer_mode == DepthBufferMode::Write,
                                depth_compare: wgpu::CompareFunction::Less,
                                stencil: wgpu::StencilState::default(),
                                bias: wgpu::DepthBiasState {
                                    constant: 2, // corresponds to bilinear filtering
                                    slope_scale: 2.0,
                                    clamp: 0.0,
                                },
                            })
                        } else {
                            None
                        },
                        multisample: wgpu::MultisampleState::default(),
                        multiview: None,
                    });

            PipelineInstance::Render(RenderPipelineBackend {
                wgpu_pipeline,
                depth_buffer_mode,
            })
        } else {
            // compute pipeline
            let wgpu_pipeline =
                ctx.device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some(&pipeline.label),
                        layout: Some(&pipeline_layout),
                        module: wgpu_shader_module,
                        entry_point: "main",
                    });

            PipelineInstance::Compute(ComputePipelineBackend { wgpu_pipeline })
        };

        Self {
            wgpu_bind_group_layouts,
            instance,
        }
    }

    fn bind_group_layout(device: &wgpu::Device, bind_group: &BindGroup) -> wgpu::BindGroupLayout {
        let entries = bind_group
            .bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| match binding {
                Binding::Uniform(_, stage, _) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: visibility(stage),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                Binding::Texture(_, stage, _) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: visibility(stage),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                Binding::Texture3D(_, stage, _) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: visibility(stage),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                    },
                    count: None,
                },
                Binding::Sampler(_, stage, _) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: visibility(stage),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                Binding::Storage(_, stage, storage) => {
                    let read_only = matches!(storage.mode, StorageBufferMode::Read);
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: visibility(stage),
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                }
                Binding::StorageAsUniform(_, stage, _) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: visibility(stage),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            })
            .collect::<Vec<_>>();

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(bind_group.label),
            entries: entries.as_slice(),
        })
    }
}

/// Pipeline Bindings
#[derive(Default)]
pub struct Bindings {
    wgpu_bind_groups: Vec<wgpu::BindGroup>,
}

impl Bindings {
    pub(crate) fn load(
        &mut self,
        ctx: &Context,
        pipeline: &PipelineBackend,
        bind_groups: &[BindGroup],
    ) {
        self.wgpu_bind_groups = pipeline
            .wgpu_bind_group_layouts
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
                                    uniform.get().as_entire_binding()
                                }
                                Binding::Texture(_, _, texture)
                                | Binding::Texture3D(_, _, texture) => {
                                    wgpu::BindingResource::TextureView(texture.get())
                                }
                                Binding::Sampler(_, _, sampler) => {
                                    wgpu::BindingResource::Sampler(sampler.get())
                                }
                                Binding::Storage(_, _, storage) => {
                                    storage.get().as_entire_binding()
                                }
                                Binding::StorageAsUniform(_, _, storage) => {
                                    storage.get().as_entire_binding()
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

/// Shader Module
#[derive(Default)]
pub struct ShaderModule {
    wgpu_shader_model: Option<wgpu::ShaderModule>,
}

impl ShaderModule {
    pub(crate) fn load(&mut self, ctx: &Context, name: &str, code: &str) {
        self.wgpu_shader_model = Some(ctx.device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
            },
        ));
    }

    /// Returns true if shader module was loaded to GPU
    pub fn loaded(&self) -> bool {
        self.wgpu_shader_model.is_some()
    }

    /// Unloads the sahder module from GPU
    pub fn unload(&mut self) {
        self.wgpu_shader_model.take();
    }

    fn get(&self) -> &wgpu::ShaderModule {
        self.wgpu_shader_model
            .as_ref()
            .expect("Shader model must be loaded")
    }
}
