use crate::assets::{ StaticModelVertex, SkinnedModelVertex, VertexAttributes };

use super::{
    bind_group_layout::{
        uniform_entry,
        texture2d_entry,
        texture3d_entry,
        sampler_entry,
    },
    widget::WidgetVertex,
};

pub struct Pipeline {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub wgpu_pipeline: wgpu::RenderPipeline,
}

#[cfg(feature = "shaderc")]
macro_rules! create_shader_module {
    ($device:expr, $name:expr, $kind:ident) => {
        #[allow(dead_code)]
        #[allow(non_upper_case_globals)]
        {
            const vert: shaderc::ShaderKind = shaderc::ShaderKind::Vertex;
            const frag: shaderc::ShaderKind = shaderc::ShaderKind::Fragment;
            let mut compiler = shaderc::Compiler::new().unwrap();
            let name = concat!($name, ".", stringify!($kind));
            let module = compiler.compile_into_spirv(
                include_str!(concat!("shaders/", $name, ".", stringify!($kind), ".glsl")),
                $kind,
                name,
                "main",
                None
            ).unwrap();

            $device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::util::make_spirv(&module.as_binary_u8()),
                flags: wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION,
            })
        }
    };
}

#[cfg(not(feature = "shaderc"))]
macro_rules! create_shader_module {
    ($device:expr, $name:expr, $kind:ident) => {
        $device.create_shader_module(
            &wgpu::include_spirv!(concat!("shaders/", $name, ".", stringify!($kind), ".spv"))
        )
    };
}

/// Pipeline for static model
impl Pipeline {

    pub fn default_for_static_model(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Pipeline {
        // TODO: custom shaders for the model to be handled here

        let shaders = (
            create_shader_module!(device, "static", vert),
            create_shader_module!(device, "static", frag),
        );

        Self::new_for_static_model(device, sc_desc, shaders)
    }

    pub fn new_for_static_model(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        (vs_module, fs_module): (wgpu::ShaderModule, wgpu::ShaderModule),
    ) -> Self {

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
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                    shader_location: 0,
                },
                // normal
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    offset: 4 * 3,
                    shader_location: 1,
                },
                // texture coordinates
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float2,
                    offset: 4 * 6,
                    shader_location: 2,
                },
            ],
        }];

        let wgpu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Static model pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: sc_desc.format,
                        color_blend: wgpu::BlendState::REPLACE,
                        alpha_blend: wgpu::BlendState::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                ],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
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
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            }),
            multisample: wgpu::MultisampleState::default(),
        });

        Pipeline {
            bind_group_layout,
            wgpu_pipeline,
        }
    }
}


/// Pipeline for skinned model
impl Pipeline {

    pub fn default_for_skinned_model(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Pipeline {
        // TODO: custom shaders for the model to be handled here

        let shaders = (
            create_shader_module!(device, "skinned", vert),
            create_shader_module!(device, "skinned", frag),
        );

        Self::new_for_skinned_model(device, sc_desc, shaders)
    }

    pub fn new_for_skinned_model(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        (vs_module, fs_module): (wgpu::ShaderModule, wgpu::ShaderModule),
    ) -> Pipeline {

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    uniform_entry(0),   // Projection * View matrix
                    uniform_entry(1),   // Model transform matrix
                    uniform_entry(2),   // pose
                    texture2d_entry(3), // texture
                    sampler_entry(4),   // sampler
                    uniform_entry(5),   // lights
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
            array_stride: SkinnedModelVertex::size(),
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                    shader_location: 0,
                },
                // normal
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    offset: 4 * 3,
                    shader_location: 1,
                },
                // texture coordinates
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float2,
                    offset: 4 * 6,
                    shader_location: 2,
                },
                // weights
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: 4 * 8,
                    shader_location: 3,
                },
                // joints
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Ushort4,
                    offset: 4 * 12,
                    shader_location: 4,
                },
            ],
        }];

        let wgpu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skinned model pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: sc_desc.format,
                        color_blend: wgpu::BlendState::REPLACE,
                        alpha_blend: wgpu::BlendState::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                ],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
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
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            }),
            multisample: wgpu::MultisampleState::default(),
        });

        Pipeline {
            bind_group_layout,
            wgpu_pipeline,
        }
    }
}

/// Pipeline for SkyBox
impl Pipeline {
    pub fn default_for_skybox(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {

        let shaders = (
            create_shader_module!(device, "skybox", vert),
            create_shader_module!(device, "skybox", frag),
        );

        Self::new_for_skybox(device, sc_desc, shaders)
    }

    pub fn new_for_skybox(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        (vs_module, fs_module): (wgpu::ShaderModule, wgpu::ShaderModule),
    ) -> Self {
        let vertex_size = std::mem::size_of::<[f32; 3]>();

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Projection * View matrix
                    uniform_entry(0),
                    // texture
                    texture3d_entry(1),
                    // sampler
                    sampler_entry(2),
                ],
            }
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                    shader_location: 0,
                },
            ],
        }];

        let wgpu_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs_module,
                    entry_point: "main",
                    targets: &[
                        wgpu::ColorTargetState {
                            format: sc_desc.format,
                            color_blend: wgpu::BlendState::REPLACE,
                            alpha_blend: wgpu::BlendState::REPLACE,
                            write_mask: wgpu::ColorWrite::ALL,
                        }
                    ],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    ..Default::default()
                },
                depth_stencil: None,
                /*Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState {
                        constant: 2, // corresponds to bilinear filtering
                        slope_scale: 2.0,
                        clamp: 0.0,
                    },
                    clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
                }),*/ 
                multisample: wgpu::MultisampleState::default(),
            }
        );

        Pipeline {
            bind_group_layout,
            wgpu_pipeline,
        }
    }
}


/// Pipeline for overlays
impl Pipeline {

    pub fn default_for_overlay(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Pipeline {
        // TODO: custom shaders for the model to be handled here

        let shaders = (
            create_shader_module!(device, "overlay", vert),
            create_shader_module!(device, "overlay", frag),
        );

        Self::new_for_overlay(device, sc_desc, shaders)
    }

    pub fn new_for_overlay(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        (vs_module, fs_module): (wgpu::ShaderModule, wgpu::ShaderModule),
    ) -> Self {

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // transform matrix
                    uniform_entry(0),
                    // texture
                    texture2d_entry(1),
                    // sampler
                    sampler_entry(2),
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
            array_stride: WidgetVertex::size(),
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float2,
                    offset: 0,
                    shader_location: 0,
                },
                // texture coordinates
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float2,
                    offset: 4 * 2,
                    shader_location: 1,
                },
                // color
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: 4 * 4,
                    shader_location: 2,
                },
            ],
        }];

        let wgpu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Overlay pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: sc_desc.format,
                        color_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            ..Default::default()
                        },
                        alpha_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            ..Default::default()
                        },
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                ],
            }),
            primitive: wgpu::PrimitiveState {
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        Pipeline {
            bind_group_layout,
            wgpu_pipeline,
        }
    }
}

