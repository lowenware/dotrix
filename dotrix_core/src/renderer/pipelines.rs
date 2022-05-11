use super::{BindGroup, Binding, Bindings, Context, GpuMesh, Renderer};
use crate::{assets::Shader, reloadable::ReloadKind};

use std::time::Instant;

/// Pipeline context
pub struct Pipeline {
    /// Gpu instance of the pipeline
    pub instance: Option<PipelineInstance>,
    /// Pipeline bindings
    pub bindings: Bindings,
    /// renderer's cycle
    pub cycle: usize,
    /// is disabled
    pub disabled: bool,
    /// Last instant that the cycle was bound
    pub last_bound_at: Instant,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self {
            instance: None,
            bindings: Default::default(),
            cycle: Default::default(),
            disabled: Default::default(),
            last_bound_at: Instant::now(),
        }
    }
}

/// Render component to control `RenderPipeline`
#[derive(Default)]
pub struct Render {
    /// Pipeline context
    pub pipeline: Pipeline,
}

/// Compute component to control `ComputePipeline`
#[derive(Default)]
pub struct Compute {
    /// Pipeline context
    pub pipeline: Pipeline,
}

impl Pipeline {
    /// Constructs new instance of `Compute` pipeline component with defined Shader
    pub fn compute() -> Compute {
        Compute {
            pipeline: Pipeline::default(),
        }
    }

    /// Constructs new instance of `Render` pipeline component with defined Shader
    pub fn render() -> Render {
        Render {
            pipeline: Pipeline::default(),
        }
    }

    /// Checks if rendering cycle should be performed
    pub fn cycle(&self, renderer: &Renderer) -> bool {
        !self.disabled && self.cycle != renderer.cycle()
    }

    /// Returns true if Pipeline is ready to run
    pub fn ready(&self, renderer: &Renderer) -> bool {
        !self.reload_required(renderer) && self.bindings.loaded()
    }

    /// Check if the instance is valid and if a reload has been requested gloabally by
    /// the renderer
    pub fn reload_required(&self, renderer: &Renderer) -> bool {
        match self.instance {
            Some(PipelineInstance::Compute(ComputePipeline { last_reload, .. }))
            | Some(PipelineInstance::Render(RenderPipeline { last_reload, .. })) => {
                last_reload < renderer.dirty
            }
            _ => true,
        }
    }

    /// Check if a bind is required based on the last time the bindings were
    /// changed and the last time this pipeline was loaded
    // TODO: Include shader in this check
    pub fn bind_required<'a, Mesh>(&self, layout: &PipelineLayout<'a, Mesh>) -> bool
    where
        Mesh: GpuMesh,
    {
        if self.instance.is_none() {
            true
        } else {
            match layout {
                PipelineLayout::Render { bindings, .. }
                | PipelineLayout::Compute { bindings, .. } => {
                    for bind_group in bindings.iter() {
                        for bind in bind_group.bindings.iter() {
                            let rebind = match bind {
                                Binding::Uniform(_, _, asset) | Binding::Storage(_, _, asset) => {
                                    matches!(
                                        asset.changes_since(self.last_bound_at),
                                        ReloadKind::Reload
                                    )
                                }
                                Binding::Texture(_, _, asset)
                                | Binding::TextureCube(_, _, asset)
                                | Binding::TextureArray(_, _, asset)
                                | Binding::Texture3D(_, _, asset)
                                | Binding::StorageTexture(_, _, asset, _)
                                | Binding::StorageTextureCube(_, _, asset, _)
                                | Binding::StorageTextureArray(_, _, asset, _)
                                | Binding::StorageTexture3D(_, _, asset, _) => {
                                    matches!(
                                        asset.changes_since(self.last_bound_at),
                                        ReloadKind::Reload
                                    )
                                }
                                Binding::Sampler(_, _, _) => false,
                            };
                            if rebind {
                                return true;
                            }
                        }
                    }
                    false
                }
            }
        }
    }
}

/// Scissors Rectangle
#[derive(Debug, Clone, Copy)]
pub struct ScissorsRect {
    /// Minimal clip size by X axis
    pub clip_min_x: u32,
    /// Minimal clip size by Y axis
    pub clip_min_y: u32,
    /// widget width
    pub width: u32,
    /// widget height
    pub height: u32,
}

/// Draw call arguments
#[derive(Debug, Clone, Copy)]
pub struct DrawArgs {
    /// Scissors Rectangle
    pub scissors_rect: Option<ScissorsRect>,
    /// Indexed draw start
    pub start_index: u32,
    /// Indexed draw end
    pub end_index: u32,
}

impl Default for DrawArgs {
    fn default() -> Self {
        Self {
            scissors_rect: None,
            start_index: 0,
            end_index: 1,
        }
    }
}

/// Compute call options
pub struct ComputeArgs {
    /// Compute work groups
    pub work_groups: WorkGroups,
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

/// Mode of the depth buffer
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DepthBufferMode {
    /// Depth buffer is disabled
    Disabled,
    /// Read + Write mode
    ReadWrite,
    /// Read Only mode
    ReadOnly,
}

/// Render Pipeline
pub struct RenderPipeline {
    /// WGPU pipeline
    pub wgpu_pipeline: wgpu::RenderPipeline,
    /// Depth Buffer Mode
    pub depth_buffer_mode: DepthBufferMode,
    /// WGPU bind group layout
    pub wgpu_bind_groups_layout: Vec<wgpu::BindGroupLayout>,
    /// Last time that a reload was done with this pipieline instance
    pub last_reload: Instant,
}

/// Compute pipeline backend
pub struct ComputePipeline {
    /// WGPU pipeline
    pub wgpu_pipeline: wgpu::ComputePipeline,
    /// WGPU bind group layout
    pub wgpu_bind_groups_layout: Vec<wgpu::BindGroupLayout>,
    /// Last time that a reload was done with this pipeline instance
    pub last_reload: Instant,
}

/// Pipeline Instance
pub enum PipelineInstance {
    /// Rendering Pipeline Instance
    Render(RenderPipeline),
    /// Compute Pipeline Instance
    Compute(ComputePipeline),
}

impl PipelineInstance {
    /// Unwrap render pipeline reference
    pub fn render(&self) -> &RenderPipeline {
        match self {
            Self::Render(pipeline) => pipeline,
            Self::Compute(_) => panic!("Compute pipeline used for rendering"),
        }
    }
    /// Unwrap compute pipeline reference
    pub fn compute(&self) -> &ComputePipeline {
        match self {
            Self::Compute(pipeline) => pipeline,
            Self::Render(_) => panic!("Render pipeline used for rendering"),
        }
    }
}

/// Pipeline layout
pub enum PipelineLayout<'a, Mesh>
where
    Mesh: GpuMesh,
{
    /// Rendering Pipeline Layout
    Render {
        /// Name of the Pipeline
        label: String,
        /// Mesh object to construct the pipeline
        mesh: &'a Mesh,
        /// Shader module
        shader: &'a Shader,
        /// Pipeline bindings
        bindings: &'a [BindGroup<'a>],
        /// Pipeline options
        options: RenderOptions<'a>,
    },
    /// Compute Pipeline Layout
    Compute {
        /// Name of the Pipeline
        label: String,
        /// Shader module
        shader: &'a Shader,
        /// Pipeline bindings
        bindings: &'a [BindGroup<'a>],
        /// Pipeline options
        options: ComputeOptions<'a>,
    },
}

impl<'a, Mesh> PipelineLayout<'a, Mesh>
where
    Mesh: GpuMesh,
    &'static Mesh: GpuMesh,
{
    /// Constructs `PipelineInstance` from the layout
    pub fn instance(&self, ctx: &Context) -> PipelineInstance {
        match self {
            PipelineLayout::Render {
                label,
                mesh,
                shader,
                bindings,
                options,
            } => PipelineLayout::render(ctx, label, mesh, shader, bindings, options),
            PipelineLayout::Compute {
                label,
                shader,
                bindings,
                options,
            } => PipelineLayout::<'a, crate::assets::Mesh>::compute(
                ctx, label, shader, bindings, options,
            ),
        }
    }

    /// Constructs Render `PipelineInstance`
    pub fn render(
        ctx: &Context,
        label: &str,
        mesh: &'a Mesh,
        shader: &Shader,
        bindings: &[BindGroup<'a>],
        options: &RenderOptions,
    ) -> PipelineInstance {
        let wgpu_shader_module = shader.module.get();
        let wgpu_bind_groups_layout = bindings
            .iter()
            .map(|bind_group| bind_group.layout(&ctx.device))
            .collect::<Vec<_>>();

        // create pipeline layout
        let wgpu_pipeline_layout =
            ctx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label),
                    bind_group_layouts: wgpu_bind_groups_layout
                        .iter()
                        .collect::<Vec<_>>()
                        .as_slice(),
                    push_constant_ranges: &[],
                });

        let depth_buffer_mode = options.depth_buffer_mode;
        // prepare vertex buffers layout
        let mut vertex_array_stride = 0;
        let vertex_attributes = mesh
            .get_vertex_buffer_layout()
            .iter()
            .enumerate()
            .map(|(index, attr)| {
                let offset = vertex_array_stride;
                vertex_array_stride += attr.size();
                wgpu::VertexAttribute {
                    format: attr.into(),
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
        let wgpu_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&wgpu_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: wgpu_shader_module,
                    entry_point: options.vs_main,
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: wgpu_shader_module,
                    entry_point: options.fs_main,
                    targets: &[wgpu::ColorTargetState {
                        format: ctx.sur_desc.format,
                        blend: match depth_buffer_mode {
                            DepthBufferMode::ReadOnly => None,
                            DepthBufferMode::ReadWrite => Some(wgpu::BlendState::ALPHA_BLENDING),
                            DepthBufferMode::Disabled => Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                            }),
                        },
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: if !options.disable_cull_mode {
                        Some(wgpu::Face::Back)
                    } else {
                        None
                    },
                    ..Default::default()
                },
                depth_stencil: if depth_buffer_mode != DepthBufferMode::Disabled {
                    Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: depth_buffer_mode == DepthBufferMode::ReadWrite,
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
                multisample: wgpu::MultisampleState {
                    count: ctx.sample_count,
                    ..Default::default()
                },
                multiview: None,
            });

        PipelineInstance::Render(RenderPipeline {
            wgpu_bind_groups_layout,
            wgpu_pipeline,
            depth_buffer_mode,
            last_reload: Instant::now(),
        })
    }

    /// Constructs Render `PipelineInstance`
    pub fn compute(
        ctx: &Context,
        label: &str,
        shader: &Shader,
        bindings: &[BindGroup<'a>],
        options: &ComputeOptions,
    ) -> PipelineInstance {
        let wgpu_shader_module = shader.module.get();
        let wgpu_bind_groups_layout = bindings
            .iter()
            .map(|bind_group| bind_group.layout(&ctx.device))
            .collect::<Vec<_>>();

        // create pipeline layout
        let wgpu_pipeline_layout =
            ctx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label),
                    bind_group_layouts: wgpu_bind_groups_layout
                        .iter()
                        .collect::<Vec<_>>()
                        .as_slice(),
                    push_constant_ranges: &[],
                });
        // compute pipeline
        let wgpu_pipeline = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: Some(&wgpu_pipeline_layout),
                module: wgpu_shader_module,
                entry_point: options.cs_main,
            });

        PipelineInstance::Compute(ComputePipeline {
            wgpu_pipeline,
            wgpu_bind_groups_layout,
            last_reload: Instant::now(),
        })
    }
}

/// Pipeline options
pub struct RenderOptions<'a> {
    /// Depth buffer mode
    pub depth_buffer_mode: DepthBufferMode,
    /// Disable cull mode
    pub disable_cull_mode: bool,
    /// Vertex Shader Entry Point
    pub vs_main: &'a str,
    /// Fragment Shader Entry Point
    pub fs_main: &'a str,
}

impl Default for RenderOptions<'_> {
    fn default() -> Self {
        Self {
            depth_buffer_mode: DepthBufferMode::ReadWrite,
            disable_cull_mode: false,
            vs_main: "vs_main",
            fs_main: "fs_main",
        }
    }
}

/// Pipeline options
pub struct ComputeOptions<'a> {
    /// Compute Shader
    pub cs_main: &'a str,
}

impl Default for ComputeOptions<'_> {
    fn default() -> Self {
        Self { cs_main: "cs_main" }
    }
}
