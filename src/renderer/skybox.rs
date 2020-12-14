use std::mem;
use wgpu::util::DeviceExt;

use crate::{
    ecs::{Const, Mut, Context},
    components::SkyBox,
    renderer::Renderer,
    services::{Assets, Camera, World},
};

pub struct RendererContext {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub proj_view: wgpu::Buffer,
    pub sampler: wgpu::Sampler,
}

impl RendererContext {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        proj_view: [f32; 32],
    ) -> Self {
        let vertex_size = mem::size_of::<[f32; 3]>();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false, filtering: true, },
                    count: None,
                },
            ],
        });

        // shaders
        #[cfg(shaderc)]
        let (vs_module, fs_module) = {
            let mut compiler = shaderc::Compiler::new().unwrap();
            let vs_spirv = compiler.compile_into_spirv(
                include_str!("shaders/skybox.vert.glsl"),
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None
            ).unwrap();

            let fs_spirv = compiler.compile_into_spirv(
                include_str!("shaders/skybox.frag.glsl"),
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None
            ).unwrap();

            (
                device.create_shader_module(&wgpu::util::make_spirv(&vs_spirv.as_binary_u8())),
                device.create_shader_module(&wgpu::util::make_spirv(&fs_spirv.as_binary_u8())),
            )
        };

        #[cfg(not(shaderc))]
        let (vs_module, fs_module) = (
            device.create_shader_module(&wgpu::include_spirv!("shaders/skybox.vert.spv")),
            device.create_shader_module(&wgpu::include_spirv!("shaders/skybox.frag.spv")),
        );

        // pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_state = wgpu::VertexStateDescriptor {
            index_format: Some(wgpu::IndexFormat::Uint16),
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
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[sc_desc.format.into()],
            depth_stencil_state: None,
            vertex_state: vertex_state.clone(),
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let proj_view = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&proj_view),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
 
        Self {
            bind_group_layout,
            pipeline,
            proj_view,
            sampler,
        }
    }
}

#[derive(Default)]
pub struct SystemContext {
    renderer_context: Option<RendererContext>,
}

/// Static Renderer system
pub fn skybox_renderer(
    mut ctx: Context<SystemContext>,
    camera: Const<Camera>,
    assets: Const<Assets>,
    renderer: Mut<Renderer>,
    world: Const<World>
) {
    let device = renderer.device();
    let sc_desc = renderer.sc_desc();
    let queue = renderer.queue();
    let frame = &renderer.frame().unwrap().output;

    let mut view = *camera.view();
    view.w.x = 0.0;
    view.w.y = 0.0;
    view.w.z = 0.0;
    // view = OPENGL_TO_WGPU_MATRIX * view;


    let mut proj_view = [0f32; 16 * 2];
    proj_view[..16].copy_from_slice(AsRef::<[f32; 16]>::as_ref(renderer.projection()));
    proj_view[16..].copy_from_slice(AsRef::<[f32; 16]>::as_ref(&view));

    let ctx = if ctx.renderer_context.is_none() {
        ctx.renderer_context = Some(RendererContext::new(device, sc_desc, proj_view));
        ctx.renderer_context.as_ref().unwrap()
    } else {
        let ctx = ctx.renderer_context.as_ref().unwrap();
        queue.write_buffer(&ctx.proj_view, 0, bytemuck::cast_slice(&proj_view));
        ctx
    };

    // render
    let query = world.query::<(&mut SkyBox,)>();

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    for (skybox,) in query {
        if skybox.buffers.is_none() {
            skybox.try_init_buffers(&assets, device, queue, ctx);
        }

        if let Some(buffers) = skybox.buffers.as_ref() {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                depth_stencil_attachment: None,
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&ctx.pipeline);
            rpass.set_bind_group(0, &buffers.bind_group, &[]);
            rpass.set_vertex_buffer(0, buffers.vertices.slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("draw indexed");
            rpass.set_index_buffer(buffers.indices.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..buffers.indices_count, 0, 0..1);
        }
    }

    queue.submit(Some(encoder.finish()));
}

