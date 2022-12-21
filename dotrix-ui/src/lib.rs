pub mod edit;
pub mod render;
pub mod text;
pub mod view;

use dotrix_core as dotrix;
use dotrix_gpu as gpu;
use dotrix_gpu::backend as wgpu;
use dotrix_log as log;

use dotrix_input::Input;
use dotrix_mesh::Mesh;
use dotrix_types::{vertex, Camera, Color, Frame, Id};
use std::ops::Range;

pub use edit::Edit;
pub use text::Text;
pub use view::View;

pub type VertexAttributes = (Position, vertex::TexUV, Color<u8>);

pub struct Overlay {
    pub rect: Rect,
    pub view: View,
}

pub struct Rect {
    pub horizontal: f32,
    pub vertical: f32,
    pub width: f32,
    pub height: f32,
}

pub struct Widget {
    pub mesh: Mesh,
    pub texture: Id<gpu::TextureView>,
}

impl Widget {
    pub fn new(mesh: Mesh, texture: Id<gpu::TextureView>) -> Widget {
        if !mesh.contains::<VertexAttributes>() {
            panic!("Widget mesh must contain Position, TexUV and Color<u8>");
        }
        Widget { mesh, texture }
    }
}

const INITIAL_VERTEX_COUNT: u64 = 4 * 64;

#[derive(Default)]
pub struct DrawTask {
    render: Option<render::Render>,
}

impl dotrix::Task for DrawTask {
    type Context = (
        dotrix::Take<dotrix::All<Overlay>>,
        dotrix::Any<Camera>,
        dotrix::Any<Frame>,
        //dotrix::Any<Input>,
        dotrix::Ref<gpu::Gpu>,
    );
    type Output = gpu::Commands;

    fn run(
        &mut self,
        (mut overlay, _camera, frame, /*input,*/ gpu): Self::Context,
    ) -> Self::Output {
        let render = self
            .render
            .get_or_insert_with(|| render::Render::new(&gpu, INITIAL_VERTEX_COUNT));

        let mut encoder = gpu.encoder(Some("dotrix::ui"));

        let (view, resolve_target) = gpu.color_attachment();

        render.write_uniform(&gpu, frame.width as f32, frame.height as f32);

        {
            let mut vertex_buffer_size: u64 = 0;
            let mut index_buffer_size: u64 = 0;

            let (vertices, indices): (Vec<_>, Vec<_>) = overlay
                .drain()
                .map(|mut entry| {
                    let Widget { mesh, .. } =
                        entry.view.compose(entry.rect, /*&input, */ &frame);
                    let vertices = mesh
                        .buffer::<VertexAttributes>()
                        .expect("Unsupported overlay mesh layout");
                    let indices =
                        Vec::from(mesh.indices::<u8>().expect("Overlay mesh MUST be indexed"));
                    vertex_buffer_size += vertices.len() as u64;
                    index_buffer_size += indices.len() as u64;
                    (vertices, indices)
                })
                .unzip();

            render.clear_vertex_buffer(&gpu, vertex_buffer_size);
            render.clear_index_buffer(&gpu, index_buffer_size);

            render.vertex_buffer.write(&gpu, &vertices);
            render.index_buffer.write(&gpu, &indices);
        }

        let slices = render
            .vertex_buffer
            .slices
            .iter()
            .zip(render.index_buffer.slices.iter());

        for (vertex_buffer_slice, index_buffer_slice) in slices {
            encoder.inner.push_debug_group("dotrix::ui::overlay");
            let mut rpass = encoder
                .inner
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

            //rpass.set_scissor_rect(rect.x, rect.y, rect.width, rect.height);

            rpass.set_pipeline(&render.render_pipeline.inner);
            rpass.set_bind_group(0, &render.bind_group, &[]);

            rpass.set_vertex_buffer(
                0,
                render
                    .vertex_buffer
                    .buffer
                    .inner
                    .slice(vertex_buffer_slice.clone()),
            );
            rpass.set_index_buffer(
                render
                    .index_buffer
                    .buffer
                    .inner
                    .slice(index_buffer_slice.clone()),
                wgpu::IndexFormat::Uint32,
            );
            let indices_count = (index_buffer_slice.end - index_buffer_slice.start)
                / std::mem::size_of::<u32>() as u64;
            rpass.draw_indexed(0..indices_count as u32, 0, 0..1);
        }

        encoder.finish(9000)
    }
}

pub struct Position {
    pub value: [f32; 2],
}

impl vertex::Attribute for Position {
    type Raw = [f32; 2];
    fn name() -> &'static str {
        "Screen Position"
    }
    fn format() -> vertex::AttributeFormat {
        vertex::AttributeFormat::Float32x2
    }
}

#[derive(Default)]
pub struct Extension {}

impl dotrix::Extension for Extension {
    fn load(&self, manager: &dotrix::Manager) {
        manager.schedule(DrawTask::default())
    }
}
