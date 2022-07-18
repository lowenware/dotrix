use crate::{Commands, Renderer};
use dotrix_core::{Color};
use dotrix_os::{Task, Any, All, Ro, Rw};

/// Clear Surface task
pub struct ClearSurface {
    pub clear_color: Color;
}

impl Default for ClearSurface {
    fn default() -> Self {
        Self {
            clear_color: Color::white(),
        }
    }
}

impl Task for ClearSurface {
    type Context = (Ro<Renderer>,);
    type Provides = Commands;

    fn run(&mut self, (renderer,): Self::Context) -> Self::Provides {
        let encoder = renderer.encoder();
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &self.reflect_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: true,
                    },
                }],
                // We still need to use the depth buffer here
                // since the pipeline requires it.
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
        Commands
    }
}
