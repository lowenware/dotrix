
mod vertex;

use dotrix_core as dotrix;
use dotrix_gpu as gpu;

use gpu::backend as wgpu;

use dotrix_math::{Vec2};
use dotrix_types::{Frame, Color};

use vertex::UVVertex;

pub struct PostProcess {
    render_priority: u32,
    vert_count: u32,
    vertex_buf: gpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: gpu::RenderPipeline,
}

impl PostProcess {
    pub fn encode(&self, gpu: &gpu::Gpu, output_view: &gpu::TextureView) -> gpu::Commands {
        let mut encoder = gpu.encoder(None);

        {
            let mut rpass = encoder.inner.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true
                    }
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline.inner);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buf.inner.slice(..));
            rpass.draw(0..self.vert_count, 0..1);
        }

        encoder.finish(self.render_priority)
    }
}

pub struct TintEffect {
    post_processing: PostProcess
}

impl TintEffect {
    pub fn new (render_priority: u32) -> Self {

        let vertices = [
            UVVertex { pos: Vec2::new(-1.0, -1.0), uv_coords: Vec2::new(0.0, 1.0) }, // 1
            UVVertex { pos: Vec2::new( 1.0, -1.0), uv_coords: Vec2::new(1.0, 1.0) }, // 2
            UVVertex { pos: Vec2::new( 1.0,  1.0), uv_coords: Vec2::new(1.0, 0.0) }, // 3
            UVVertex { pos: Vec2::new( 1.0,  1.0), uv_coords: Vec2::new(1.0, 0.0) }, // 3
            UVVertex { pos: Vec2::new(-1.0,  1.0), uv_coords: Vec2::new(0.0, 0.0) }, // 4
            UVVertex { pos: Vec2::new(-1.0, -1.0), uv_coords: Vec2::new(0.0, 1.0) }, // 1
        ];

        Self {
            post_processing: PostProcess {
                render_priority,
                vert_count: vertices.len() as u32,
                vertex_buf,
                bind_group,
                pipeline,
            },
        }
    }
}

impl dotrix::Task for TintEffect {
    type Context = (
        dotrix::Ref<gpu::Gpu>,
        dotrix::Any<Frame>
    );

    type Output = gpu::Commands;

    fn run (&mut self, (gpu, frame): Self::Context) -> Self::Output {
        self.post_processing.encode(&gpu, &frame)
    }
}

pub struct Extension {

}

impl dotrix::Extension for Extension {
    fn load(&self, manager: &dotrix::Manager) {
        let tint_effect_task = TintEffect::new(5000);
        manager.schedule(tint_effect_task);
    }
}