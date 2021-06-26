//! Constructors for rendering pipelines
use std::borrow::Cow;

use crate::assets::{
    StaticModelVertex,
    SkinnedModelVertex,
    VertexAttributes,
    WireFrameVertex
};

use super::{
    bind_group_layout::{
        uniform_entry,
        texture2d_entry,
        texture3d_entry,
        sampler_entry,
    },
    widget::WidgetVertex,
};

/// Rendering Pipeline
pub struct Pipeline {
    /// WGPU bind group layout
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// WGPU pipeline
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

    /// Returns pipeline for [`crate::components::Model`] without a skin with default shaders
    pub fn default_for_static_model(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Pipeline {
        let shaders = (
            create_shader_module!(device, "static", vert),
            create_shader_module!(device, "diffuse", frag),
        );

        Self::new_for_static_model(device, sc_desc, shaders)
    }

    /// Returns pipeline for [`crate::components::Model`] without a skin with custom shaders
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
}


/// Pipeline for skinned model
impl Pipeline {

    /// Returns pipeline for [`crate::components::Model`] with skin and default shaders
    pub fn default_for_skinned_model(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Pipeline {
        let shaders = (
            create_shader_module!(device, "skinned", vert),
            create_shader_module!(device, "diffuse", frag),
        );

        Self::new_for_skinned_model(device, sc_desc, shaders)
    }

    /// Returns pipeline for [`crate::components::Model`] with skin and custom shaders
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
                // weights
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 8,
                    shader_location: 3,
                },
                // joints
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint16x4,
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
}

/// Pipeline for SkyBox
impl Pipeline {
    /// Returns pipeline for [`crate::components::SkyBox`] with default shaders
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

    /// Returns pipeline for [`crate::components::SkyBox`] with custom shaders
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
                    format: wgpu::VertexFormat::Float32x3,
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
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrite::ALL,
                        }
                        /* wgpu::ColorTargetState {
                            format: sc_desc.format,
                            color_blend: wgpu::BlendState::REPLACE,
                            alpha_blend: wgpu::BlendState::REPLACE,
                            write_mask: wgpu::ColorWrite::ALL,
                        } */
                    ],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
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

    /// Returns pipeline for [`crate::renderer::OverlayProvider`] with default shaders
    pub fn default_for_overlay(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Pipeline {
        let shaders = (
            create_shader_module!(device, "overlay", vert),
            create_shader_module!(device, "overlay", frag),
        );

        Self::new_for_overlay(device, sc_desc, shaders)
    }

    /// Returns pipeline for [`crate::renderer::OverlayProvider`] with custom shaders
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
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                // texture coordinates
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 2,
                    shader_location: 1,
                },
                // color
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
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
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                ..Default::default()
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                dst_factor: wgpu::BlendFactor::One,
                                ..Default::default()
                            },
                        }),
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                    /*
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
                    */
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


/// Pipeline for static model
impl Pipeline {

    /// Returns pipeline for [`crate::components::WireFrame`] with default shaders
    pub fn default_for_wire_frame(
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
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/wires.wgsl"))),
            flags,
        });

        Self::new_for_wire_frame(device, sc_desc, shader)
    }

    /// Returns pipeline for [`crate::components::WireFrame`] with custom shaders
    pub fn new_for_wire_frame(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        shader: wgpu::ShaderModule,
    ) -> Self {

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Projection * View matrix
                    uniform_entry(0),
                    // Model transform matrix
                    uniform_entry(1),
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
            array_stride: WireFrameVertex::size(),
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                // color
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 4 * 3,
                    shader_location: 1,
                },
            ],
        }];

        let wgpu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("WireFrame pipeline"),
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
                topology: wgpu::PrimitiveTopology::LineList,
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
}
