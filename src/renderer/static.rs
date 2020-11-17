use std::mem;
use wgpu::util::DeviceExt;

use crate::{
    assets::{ Assets, Id, Texture, Mesh, Vertex },
    camera::Camera,
    ecs::{ Const, Mut, Context },
    renderer::{ Light, LightUniform, Renderer },
    World
};

const VERTEXT_SHADER: &str = include_str!("shaders/static.vert.glsl");
const FRAGMENT_SHADER: &str = include_str!("shaders/static.frag.glsl");

/// Static Renderer component
pub struct StaticModel {
    mesh: Id<Mesh>,
    texture: Id<Texture>,
    // transform: cgmath::Matrix4<f32>,
    vertices_buffer: Option<wgpu::Buffer>,
    indices_buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
    indices_count: usize,
}

impl StaticModel {
    pub const fn new(mesh: Id<Mesh>, texture: Id<Texture>) -> Self {
        // use cgmath::SquareMatrix;
        Self {
            mesh,
            texture,
            vertices_buffer: None,
            indices_buffer: None,
            bind_group: None,
            indices_count: 0,
            // transform: cgmath::Matrix4::identity(),
        }
    }
}

#[derive(Default)]
pub struct StaticRenderer {
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    pipeline: Option<wgpu::RenderPipeline>,
    lights_buffer: Option<wgpu::Buffer>,
    uniform_buffer: Option<wgpu::Buffer>, // Projection + View + Transfromation
}

/// Static Renderer system
pub fn static_renderer(
    mut ctx: Context<StaticRenderer>,
    camera: Const<Camera>,
    assets: Const<Assets>,
    renderer: Mut<Renderer>,
    world: Const<World>
) {
    let device = renderer.device();
    let sc_desc = renderer.sc_desc();
    let queue = renderer.queue();
    let frame = &renderer.frame().unwrap().output;
    let depth_buffer = renderer.depth_buffer();

    let vertex_size = mem::size_of::<Vertex>();

    // PVM (Projection * View * Model) matrix unfiorm
    // TODO: There will be 3 matrices:
    // 1. projection: can be changed on window resize
    // 2. view: can be chaged by interactions with camera
    // 3. transform: model related, can be changed by various systems
    // Each matrix should be available in shaders through uniform variables and their buffers
    // must be updated on change (transform matrix will be updated inside of the ECS query
    // loop
    let mx_total = renderer.projection() * camera.view();
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    if ctx.uniform_buffer.is_none() {
        ctx.uniform_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        }));
    } else {
        let queue = renderer.queue();
        queue.write_buffer(ctx.uniform_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(mx_ref));
    }

    if ctx.pipeline.is_none() {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                },
                // Depth texture
                /* wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },*/
            ],
        });

        ctx.bind_group_layout = Some(bind_group_layout);

        // shaders
        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler.compile_into_spirv(
            VERTEXT_SHADER,
            shaderc::ShaderKind::Vertex,
            "shader.vert",
            "main",
            None
        ).unwrap();
        let fs_spirv = compiler.compile_into_spirv(
            FRAGMENT_SHADER,
            shaderc::ShaderKind::Fragment,
            "shader.frag",
            "main",
            None
        ).unwrap();

        let vs_module = device.create_shader_module(
            wgpu::util::make_spirv(&vs_spirv.as_binary_u8()));
        let fs_module = device.create_shader_module(
            wgpu::util::make_spirv(&fs_spirv.as_binary_u8()));

        // pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[ctx.bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        let vertex_state = wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: vertex_size as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    // position
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 0,
                        shader_location: 0,
                    },
                    // normal
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 4 * 3,
                        shader_location: 1,
                    },
                    // texture coordinates
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 4 * 6,
                        shader_location: 2,
                    },
                ],
            }],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: vertex_state.clone(),
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        ctx.pipeline = Some(pipeline);
    }

    // light

    let query = world.query::<(&mut Light,)>();
    let mut lights = LightUniform::default();
    for (light,) in query {
        lights.push(light.clone());
    }

    if ctx.lights_buffer.is_none() {
        ctx.lights_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[lights]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        ));
    } else {
        let queue = renderer.queue();
        queue.write_buffer(ctx.lights_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&[lights]));
    }

    // render
    let query = world.query::<(&mut StaticModel,)>();

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // TODO: consider moving to standalone renderer?
    {
        let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: depth_buffer,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
    }

    for (model,) in query {

        if model.bind_group.is_none() {
            if let Some(texture_asset) = assets.get::<Texture>(model.texture) {
                // Texture
                let texture_extent = wgpu::Extent3d {
                    width: texture_asset.width,
                    height: texture_asset.height,
                    depth: texture_asset.depth,
                };
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: texture_extent,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
                });
                let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                queue.write_texture(
                    wgpu::TextureCopyView {
                        texture: &texture,
                        mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    },
                    &texture_asset.data,
                    wgpu::TextureDataLayout {
                        offset: 0,
                        bytes_per_row: 4 * texture_asset.width,
                        rows_per_image: 0,
                    },
                    texture_extent,
                );

                // sampler
                let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
               });

                // Create bind group
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: ctx.bind_group_layout.as_ref().unwrap(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: ctx.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: ctx.lights_buffer.as_ref().unwrap().as_entire_binding(),
                        },
                        /* wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(ctx.depth_buffer.as_ref().unwrap()),
                        }, */
                    ],
                    label: None,
                });

                model.bind_group = Some(bind_group);
            }
        }

        if model.vertices_buffer.is_none() {
            if let Some(mesh) = assets.get::<Mesh>(model.mesh) {
                let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&mesh.vertices),
                    usage: wgpu::BufferUsage::VERTEX,
                });
                model.vertices_buffer = Some(vertex_buf);

                let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsage::INDEX,
                });
                model.indices_buffer = Some(index_buf);
                model.indices_count = mesh.indices.len();
            }
        }

        if let Some(vertices_buffer) = model.vertices_buffer.as_ref() {
            if let Some(bind_group) = model.bind_group.as_ref() {
               let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load, 
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: depth_buffer,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            }),
                            stencil_ops: None,
                        }
                    ),
                });
                rpass.push_debug_group("Prepare data for draw.");
                rpass.set_pipeline(ctx.pipeline.as_ref().unwrap());
                rpass.set_bind_group(0, bind_group, &[]);
                rpass.set_index_buffer(model.indices_buffer.as_ref().unwrap().slice(..));
                rpass.set_vertex_buffer(0, vertices_buffer.slice(..));
                rpass.pop_debug_group();
                rpass.insert_debug_marker("Draw!");
                rpass.draw_indexed(0..model.indices_count as u32, 0, 0..1);
            }
        }
    }

    queue.submit(Some(encoder.finish()));

}

