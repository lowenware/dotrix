use std::borrow::Cow;
use wgpu;

use dotrix_core::{
    assets::{
        StaticModelVertex,
        VertexAttributes,
    },
    renderer::{
        Pipeline,
        bind_group_layout::{
            uniform_entry,
            texture2d_entry,
            texture3d_entry,
           sampler_entry,
        },
    },
};

/// Returns pipeline for [`crate::components::Model`] without a skin with default shaders
pub fn default_pipeline(
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> Pipeline {

    let mut flags = wgpu::ShaderFlags::VALIDATION;
    match adapter.get_info().backend {
        wgpu::Backend::Metal | wgpu::Backend::Vulkan => {
            flags |= wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION
        }
        _ => (),
    }
    println!("Create shader module: begin");
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Terrain Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/terrain.wgsl"))),
        flags,
    });
    println!("Create shader module: end");

    new_pipeline(device, sc_desc, shader)
}

/// Returns pipeline for [`crate::components::Model`] without a skin with custom shaders
pub fn new_pipeline(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    shader: wgpu::ShaderModule,
) -> Pipeline {

    let bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Projection * View matrix
                uniform_entry(0),
                // Model transform matrix
                uniform_entry(1),
                // texture
                texture2d_entry(3),
                // sampler
                sampler_entry(4),
                // lights
                uniform_entry(5),
            ],
        }
    );

        let pipeline_layout = device.create_pipeline_layout(
         &wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        }
    );

    let vertex_buffers = [wgpu::VertexBufferLayout {
        array_stride: StaticModelVertex::size(),
        step_mode: wgpu::InputStepMode::Vertex,
        attributes: &[
            // position
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            // normal
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 4 * 3,
                shader_location: 1,
            },
            // texture coordinates
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 4 * 6,
                shader_location: 2,
            },
        ],
    }];

    let wgpu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Static model pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &vertex_buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[
                wgpu::ColorTargetState {
                    format: sc_desc.format,
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
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2, // corresponds to bilinear filtering
                slope_scale: 2.0,
                clamp: 0.0,
            },
            // clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
        }),
        multisample: wgpu::MultisampleState::default(),
    });

    Pipeline {
        bind_group_layout,
        wgpu_pipeline,
    }
}

