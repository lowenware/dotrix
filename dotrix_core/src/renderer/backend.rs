/// WGPU backend wrapper module
use std::borrow::Cow;
use wgpu;
use wgpu::util::DeviceExt;
use winit;

use crate::{
    assets::Shader,
    generics::{ Id, Color },
};

use super::{
    AttributeFormat,
    BindGroup,
    BindGroupLayout,
    Binding,
    PipelineLayout
};

pub(crate) struct Context {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,
    depth_buffer: wgpu::TextureView,
    frame: Option<wgpu::SwapChainFrame>,
    encoder: Option<wgpu::CommandEncoder>,
}

impl Context {
    pub(crate) fn bind_frame(&mut self, clear_color: &Color) {
        let frame = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
                self.swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture!")
            }
        };

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor { label: None };
        let mut encoder = self.device.create_command_encoder(&command_encoder_descriptor);
        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.output.view,
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
        self.frame.take();
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.sc_desc.width = width;
            self.sc_desc.height = height;

            self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
            self.depth_buffer = create_depth_buffer(&self.device, width, height);
        }
    }
}

pub(crate) async fn init(window: &winit::window::Window) -> Context {
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropiate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None, // Some(&std::path::Path::new("./wgpu-trace/")),
        )
        .await
        .expect("Failed to create device");

    let size = window.inner_size();

    let sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let depth_buffer = create_depth_buffer(&device, size.width, size.height);
    let swap_chain = device.create_swap_chain(&surface, &sc_desc);

    Context {
        adapter,
        device,
        queue,
        surface,
        swap_chain,
        sc_desc,
        depth_buffer,
        frame: None,
        encoder: None,
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
        usage: wgpu::TextureUsage::SAMPLED
            | wgpu::TextureUsage::COPY_DST
            | wgpu::TextureUsage::RENDER_ATTACHMENT,
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
            self.attributes = Some(
                ctx.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("VertexBuffer"),
                        contents: attributes,
                        usage: wgpu::BufferUsage::VERTEX,
                    }
                )
            );
        }

        if let Some(buffer) = self.indices.as_ref() {
            let indices = indices.expect("Indexed meshed can't be reloaded without indices");
            ctx.queue.write_buffer(buffer, 0, indices);
        } else {
            self.indices = indices.map(|contents| {
                ctx.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("IndexBuffer"),
                        contents,
                        usage: wgpu::BufferUsage::INDEX,
                    }
                )
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
        self.attributes.as_ref().expect("Attributes buffer must be loaded")
    }

    fn indices(&self) -> Option<&wgpu::Buffer> {
        self.indices.as_ref()
    }
}


#[derive(Default)]
pub struct TextureBuffer {
    wgpu_texture_view: Option<wgpu::TextureView>,
}

impl TextureBuffer {
    /// Loads data into the texture buffer
    pub(crate) fn load<'a>(
        &mut self,
        ctx: &Context,
        width: u32,
        height: u32,
        data: &'a [u8],
    ) {
        let texture_extent = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TextureBuffer"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        self.wgpu_texture_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));

        let bytes_per_row = std::num::NonZeroU32::new(data.len() as u32 / height)
            .unwrap();

        ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(std::num::NonZeroU32::new(height).unwrap()),
            },
            texture_extent,
        );
    }

    /// Checks if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.wgpu_texture_view.is_none()
    }

    /// Release all resources used by the buffer
    pub fn empty(&mut self) {
        self.wgpu_texture_view.take();
    }

    fn get(&self) -> &wgpu::TextureView {
        self.wgpu_texture_view.as_ref().expect("Texture must be loaded")
    }
}


#[derive(Default)]
pub struct UniformBuffer {
    wgpu_buffer: Option<wgpu::Buffer>,
}

impl UniformBuffer {
    /// Loads data into the uniform buffer
    pub(crate) fn load<'a>(
        &mut self,
        ctx: &Context,
        data: &'a [u8],
    ) {
        if let Some(buffer) = self.wgpu_buffer.as_ref() {
            ctx.queue.write_buffer(buffer, 0, data);
        } else {
            self.wgpu_buffer = Some(
                ctx.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("UniformBuffer"),
                        contents: data,
                        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                    }
                )
            );
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
        self.wgpu_buffer.as_ref().expect("Uniform buffer must be loaded")
    }
}


#[derive(Default)]
pub struct Sampler {
    wgpu_sampler: Option<wgpu::Sampler>,
}

impl Sampler {
    /// Loads the Sampler
    pub(crate) fn load(&mut self, ctx: &Context) {
        if self.wgpu_sampler.is_some() { return; }
        self.wgpu_sampler = Some(ctx.device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        ));
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


pub struct PipelineBackend {
    /// WGPU bind group layout
    wgpu_bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    /// WGPU pipeline
    wgpu_pipeline: wgpu::RenderPipeline,
}

#[inline(always)]
fn visibility(vertex: bool, fragment: bool) -> wgpu::ShaderStage {
    let mut result = wgpu::ShaderStage::NONE;

    if vertex { 
        result |= wgpu::ShaderStage::VERTEX;
    }

    if fragment {
        result |= wgpu::ShaderStage::FRAGMENT;
    }

    result
}

impl PipelineBackend {
    pub(crate) fn new(ctx: &Context, pipeline: &PipelineLayout, shader_module: &ShaderModule) -> Self {

        let wgpu_shader_module = shader_module.get();
        let wgpu_bind_group_layouts = pipeline.layout.iter()
            .map(|bind_group_layout| Self::bind_group_layout(&ctx.device, bind_group_layout))
            .collect::<Vec<_>>();

        // create pipeline layout
        let pipeline_layout = ctx.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: wgpu_bind_group_layouts.iter()
                    .map(|layout| layout)
                    .collect::<Vec<_>>()
                    .as_slice(),
                push_constant_ranges: &[],
            }
        );

        // prepare vertex buffers layout
        let mut vertex_array_stride = 0;
        let vertex_attributes = pipeline.vertex_attributes.as_ref()
            .expect(
                "Only render pipelines are currently supported, so vertex attributes are mandatory"
            ).iter()
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
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: vertex_attributes.as_slice(),
        }];

        // create the pipeline
        let wgpu_pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&pipeline.label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &wgpu_shader_module,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &wgpu_shader_module,
                entry_point: "fs_main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: ctx.sc_desc.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                ],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: if pipeline.use_depth_buffer {
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState {
                        constant: 2, // corresponds to bilinear filtering
                        slope_scale: 2.0,
                        clamp: 0.0,
                    },
                })
            } else { None },
            multisample: wgpu::MultisampleState::default(),
        });

        Self {
            wgpu_bind_group_layouts,
            wgpu_pipeline,
        }
    }

    fn bind_group_layout(
        device: &wgpu::Device,
        bind_group_layout: &BindGroupLayout
    ) -> wgpu::BindGroupLayout {
        let entries = bind_group_layout.bindings.iter()
            .enumerate()
            .map(|(index, binding)| match binding {
                Binding::Uniform(binding_type) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: visibility(binding_type.vertex, binding_type.fragment),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                Binding::Texture(binding_type) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: visibility(binding_type.vertex, binding_type.fragment),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                Binding::Sampler(binding_type) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: visibility(binding_type.vertex, binding_type.fragment),
                        ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            }).collect::<Vec<_>>();

        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some(&bind_group_layout.label),
                entries: entries.as_slice(),
            }
        )
    }

    pub(crate) fn run(
        &self,
        ctx: &mut Context,
        vertex_buffer: &VertexBuffer,
        bindings: &Bindings
    ) {
        let encoder = ctx.encoder.as_mut().expect("WGPU encoder must be set");
        let frame = ctx.frame.as_ref().expect("WGPU frame must be set");
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, 
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: &ctx.depth_buffer,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }
            ),
        });

        rpass.push_debug_group("Prepare to run pipeline");
        rpass.set_pipeline(&self.wgpu_pipeline);
        for (index, wgpu_bind_group) in bindings.wgpu_bind_groups.iter().enumerate() {
            rpass.set_bind_group(index as u32, wgpu_bind_group, &[]);
        }
        rpass.set_vertex_buffer(0, vertex_buffer.get().slice(..));
        rpass.pop_debug_group();

        let count = vertex_buffer.count;

        if let Some(indices_buffer) = vertex_buffer.indices().as_ref() {
            rpass.insert_debug_marker("Draw indexed");
            rpass.set_index_buffer(indices_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..count, 0, 0..1);
        } else {
            rpass.insert_debug_marker("Draw");
            rpass.draw(0..count, 0..1);
        }
    }
}


#[derive(Default)]
pub struct Bindings {
    wgpu_bind_groups: Vec<wgpu::BindGroup>,
}


impl Bindings {
    pub(crate) fn load(
        &mut self,
        ctx: &Context,
        pipeline: &PipelineBackend,
        bind_groups: &[BindGroup]
    ) {
        self.wgpu_bind_groups = pipeline.wgpu_bind_group_layouts.iter()
            .enumerate()
            .map(|(group, wgpu_bind_group_layout)| ctx.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: wgpu_bind_group_layout,
                    entries: bind_groups[group].bindings.iter()
                        .enumerate()
                        .map(|(binding, entry)| wgpu::BindGroupEntry {
                            binding: binding as u32,
                            resource: match entry {
                                Binding::Uniform(uniform) =>
                                    uniform.get().as_entire_binding(),
                                Binding::Texture(texture) =>
                                    wgpu::BindingResource::TextureView(texture.get()),
                                Binding::Sampler(sampler) =>
                                    wgpu::BindingResource::Sampler(sampler.get()),
                            }
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                        label: None,
                }
            ))
            .collect::<Vec<_>>();
    }

    pub fn loaded(&self) -> bool {
        self.wgpu_bind_groups.len() > 0
    }

    pub fn unload(&mut self) {
        self.wgpu_bind_groups.clear();
    }
}


#[derive(Default)]
pub struct ShaderModule {
    wgpu_shader_model: Option<wgpu::ShaderModule>,
}

impl ShaderModule {
    pub(crate) fn load(&mut self, ctx: &Context, name: &str, code: &str) {
        let mut flags = wgpu::ShaderFlags::VALIDATION;
        match ctx.adapter.get_info().backend {
            wgpu::Backend::Metal | wgpu::Backend::Vulkan => {
                flags |= wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION
            }
            _ => (),
        }
        self.wgpu_shader_model = Some(ctx.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
            flags,
        }));
    }

    pub fn is_loaded(&self) -> bool {
        self.wgpu_shader_model.is_some()
    }

    pub fn unload(&mut self) {
        self.wgpu_shader_model.take();
    }

    fn get(&self) -> &wgpu::ShaderModule {
        self.wgpu_shader_model.as_ref().expect("Shader model must be loaded")
    }
}
