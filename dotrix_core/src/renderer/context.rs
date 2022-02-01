/// WGPU backend wrapper module
use std::collections::HashMap;
use wgpu;
use winit;

use crate::assets::{Mesh, Shader};
use crate::{Color, Id};

use super::{Bindings, ComputeArgs, DepthBufferMode, DrawArgs, PipelineInstance};

/// Renderer Context
pub struct Context {
    /// WGPU Adapter
    pub adapter: wgpu::Adapter,
    /// WGPU Device
    pub device: wgpu::Device,
    /// WGPU Queue
    pub queue: wgpu::Queue,
    /// WGPU Surface
    pub surface: wgpu::Surface,
    /// WGPU Surface Configuration
    pub sur_desc: wgpu::SurfaceConfiguration,
    /// Depth Buffer implementation
    pub depth_buffer: wgpu::TextureView,
    /// Frame Surface Texture
    pub frame: Option<wgpu::SurfaceTexture>,
    /// WGPU command encoder
    pub encoder: Option<wgpu::CommandEncoder>,
    /// List of Pipeline Instances
    pub pipelines: HashMap<Id<Shader>, PipelineInstance>,
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

    pub(crate) fn add_pipeline(&mut self, shader: Id<Shader>, pipeline_instance: PipelineInstance) {
        self.pipelines.insert(shader, pipeline_instance);
    }

    pub(crate) fn has_pipeline(&self, shader: Id<Shader>) -> bool {
        self.pipelines.contains_key(&shader)
    }

    pub(crate) fn pipeline(&self, shader: Id<Shader>) -> Option<&PipelineInstance> {
        self.pipelines.get(&shader)
    }

    pub(crate) fn run_render_pipeline(
        &mut self,
        shader: Id<Shader>,
        mesh: &Mesh,
        bindings: &Bindings,
        args: &DrawArgs,
    ) {
        if let Some(pipeline_instance) = self.pipelines.get(&shader) {
            let render_pipeline = pipeline_instance.render();
            let depth_buffer_mode = render_pipeline.depth_buffer_mode;
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
            rpass.set_pipeline(&render_pipeline.wgpu_pipeline);

            if let Some(scissors_rect) = args.scissors_rect.as_ref() {
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
            rpass.set_vertex_buffer(0, mesh.vertex_buffer.get().slice(..));
            rpass.pop_debug_group();

            let count = mesh.count_vertices();

            if let Some(index_buffer) = mesh.index_buffer.as_ref() {
                rpass.insert_debug_marker("Draw indexed");
                rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..count, 0, args.start_index..args.end_index);
            } else {
                rpass.insert_debug_marker("Draw");
                rpass.draw(0..count, args.start_index..args.end_index);
            }
        }
    }

    pub(crate) fn run_compute_pipeline(
        &mut self,
        shader: Id<Shader>,
        bindings: &Bindings,
        args: &ComputeArgs,
    ) {
        if let Some(pipeline_instance) = self.pipelines.get(&shader) {
            let compute_pipeline = pipeline_instance.compute();
            let encoder = self.encoder.as_mut().expect("WGPU encoder must be set");

            // compute pass
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&compute_pipeline.wgpu_pipeline);
            for (index, wgpu_bind_group) in bindings.wgpu_bind_groups.iter().enumerate() {
                cpass.set_bind_group(index as u32, wgpu_bind_group, &[]);
            }
            cpass.dispatch(args.work_groups.x, args.work_groups.y, args.work_groups.z);
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
