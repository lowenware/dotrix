// pub mod composer;
pub mod context;
pub mod edit;
pub mod font;
pub mod overlay;
pub mod render;
pub mod style;
pub mod text;
pub mod view;

use std::collections::HashMap;
use std::time::Instant;

use dotrix_core as dotrix;
use dotrix_gpu as gpu;
use dotrix_gpu::backend as wgpu;
use dotrix_log as log;

use dotrix_input::Input;
use dotrix_types::{Camera, Frame, Id};

pub use edit::Edit;
pub use overlay::{Overlay, Rect, Widget};
pub use style::{Direction, Style};
pub use text::Text;
pub use view::View;

// use composer::Composer;

const INITIAL_VERTEX_COUNT: u64 = 4 * 64;

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Text(String),
    Number(i64),
    Decimal(f64),
}

impl Default for Value {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone)]
pub struct State {
    last_change: Instant,
    vertical_offset: f32,
    horizontal_offset: f32,
    value: Value,
    hovered: bool,
    has_focus: bool,
}

impl State {
    pub fn update(&mut self, rect: &Rect, input: &Input) {
        log::warn!("State::update is not implemented yet")
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            last_change: Instant::now(),
            vertical_offset: 0.0,
            horizontal_offset: 0.0,
            value: Value::None,
            hovered: false,
            has_focus: false,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Viewport {
    pub rect: Rect,
    pub content_width: f32,
    pub content_height: f32,
    pub direction: Direction,
}

impl Viewport {
    pub fn new(rect: Rect, direction: Direction) -> Self {
        Self {
            rect,
            direction,
            content_width: 0.0,
            content_height: 0.0,
        }
    }

    /// Returns next child empty `Rect`
    pub fn place_content(&self) -> Rect {
        match self.direction {
            Direction::Vertical => Rect {
                horizontal: self.rect.horizontal,
                vertical: self.rect.vertical + self.content_height,
                width: self.rect.width,
                height: self.rect.height - self.content_height,
            },
            Direction::Horizontal => Rect {
                horizontal: self.rect.horizontal + self.content_width,
                vertical: self.rect.vertical,
                width: self.rect.width - self.content_width,
                height: self.content_height,
            },
        }
    }

    /// Update the view port with child
    pub fn append(&mut self, rect: &Rect) {
        match self.direction {
            Direction::Vertical => {
                self.content_height += rect.height;
                if self.content_width < rect.width {
                    self.content_width = rect.width;
                }
            }
            Direction::Horizontal => {
                self.content_width += rect.width;
                if self.content_height < rect.height {
                    self.content_height = rect.height;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Context {
    states: HashMap<String, State>,
    frame_width: f32,
    frame_height: f32,
    scale_factor: f32,
    stack: Vec<Viewport>,
    widgets: Vec<overlay::Widget>,
    input: Input,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, input: &Input, frame: &Frame) {
        self.input = input.clone();
        self.frame_width = frame.width as f32;
        self.frame_height = frame.height as f32;
        self.scale_factor = frame.scale_factor;
    }

    pub fn view<F>(&mut self, id: Option<&str>, style: &Style, callback: F)
    where
        F: FnOnce(&mut Context),
    {
        // Get parent viewport
        let viewport = self
            .stack
            .last()
            .cloned()
            .expect("Overlay stack MUST always contain a viewport");

        // Calculate rectangle of the next child, taking in account existing content of parent
        let rect = viewport.place_content();
        let mut view = View::new(rect.clone(), style);

        // Get viewport with empty content of the current view, taking in account
        // its internal direction
        self.stack.push(view.viewport());

        // Run callback for children
        callback(self);

        // Retrieve the viewport with updated content size
        let viewport = self
            .stack
            .pop()
            .expect("The owner view MUST pop out its viewport");

        // Update view's rectangle, because now we know its content size
        view.update_size(viewport.content_width, viewport.content_height);

        // Update View's state with user inputs
        let state = id.map(|id| {
            let state = self.states.get(id);
            let state = state.cloned();
            let mut state = state.unwrap_or_default();

            // Here must be handled all the inputs
            state.update(&view.inner, &self.input);

            self.states.insert(String::from(id), state.clone());
            state
        });

        // Update parent Viewport's content size with size of this View
        {
            let viewport = self.stack.last_mut().expect("Parent viewport must exists");
            viewport.append(&view.outer);
        }
        // If the view has renderable content, make a widget from it
        let frame_width = self.frame_width;
        let frame_height = self.frame_height;
        let scale_factor = self.scale_factor;
        if let Some(widget) = view.compose(state, frame_width, frame_height, scale_factor) {
            self.widgets.push(widget);
        }
    }

    // pub fn label(&mut self, id: Option<&str>, style: &FontStyle, text: impl Into<String>) {
    //     let label = Label::new(id, style, text);
    // }

    pub fn overlay<F>(&mut self, rect: Rect, callback: F) -> Overlay
    where
        F: FnOnce(&mut Context),
    {
        let viewport = Viewport::new(rect, Direction::Vertical);

        self.stack.clear();
        self.widgets.clear();
        self.stack.push(viewport);

        callback(self);

        self.stack.pop();

        let widgets_len = self.widgets.len();
        let widgets = self.widgets.drain(0..widgets_len).collect::<Vec<_>>();

        Overlay { widgets }
    }
}

pub struct DrawTask {
    render: Option<render::Render>,
    texture_bind_groups: HashMap<Id<gpu::TextureView>, wgpu::BindGroup>,
}

impl dotrix::Task for DrawTask {
    type Context = (
        dotrix::Take<dotrix::All<Overlay>>,
        dotrix::Any<Camera>,
        dotrix::Any<Frame>,
        dotrix::Any<Input>,
        dotrix::Ref<gpu::Gpu>,
    );
    type Output = gpu::Commands;

    fn run(
        &mut self,
        (mut overlay_collection, _camera, frame, input, gpu): Self::Context,
    ) -> Self::Output {
        let render = self
            .render
            .get_or_insert_with(|| render::Render::new(&gpu, INITIAL_VERTEX_COUNT));

        let mut encoder = gpu.encoder(Some("dotrix::ui"));

        let (view, resolve_target) = gpu.color_attachment();

        render.write_uniform(&gpu, frame.width as f32, frame.height as f32);

        for mut overlay in overlay_collection.drain() {
            let mut vertex_buffer_size: u64 = 0;
            let mut index_buffer_size: u64 = 0;
            let (drawings, vertices, indices) = {
                let widgets_len = overlay.widgets.len();
                let mut drawings = Vec::with_capacity(widgets_len);
                let (vertices, indices): (Vec<_>, Vec<_>) = overlay
                    .widgets
                    .drain(0..widgets_len)
                    .rev()
                    .map(|widget| {
                        let vertices = widget
                            .mesh
                            .buffer::<overlay::VertexAttributes>()
                            .expect("Unsupported overlay mesh layout");
                        let indices = Vec::from(
                            widget
                                .mesh
                                .indices::<u8>()
                                .expect("Overlay mesh MUST be indexed"),
                        );
                        drawings.push((widget.texture, widget.rect.clone()));
                        vertex_buffer_size += vertices.len() as u64;
                        index_buffer_size += indices.len() as u64;

                        (vertices, indices)
                    })
                    .unzip();
                (drawings, vertices, indices)
            };

            render.clear_vertex_buffer(&gpu, vertex_buffer_size);
            render.clear_index_buffer(&gpu, index_buffer_size);

            render.vertex_buffer.write(&gpu, &vertices);
            render.index_buffer.write(&gpu, &indices);

            let drawings = drawings.into_iter().zip(
                render
                    .vertex_buffer
                    .slices
                    .iter()
                    .zip(render.index_buffer.slices.iter()),
            );

            for ((texture_id, rect), (vertex_buffer_slice, index_buffer_slice)) in drawings {
                let texture_bind_group = if texture_id.is_null() {
                    &render.default_texture_bind_group
                } else {
                    if self.texture_bind_groups.get(&texture_id).is_none() {
                        if let Some(texture) = gpu.get(&texture_id) {
                            let texture_bind_group =
                                render.create_texture_bind_group(&gpu, texture);
                            self.texture_bind_groups
                                .insert(texture_id, texture_bind_group);
                        } else {
                            continue;
                        }
                    }
                    self.texture_bind_groups.get(&texture_id).unwrap()
                };

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

                rpass.set_scissor_rect(
                    rect.horizontal.round() as u32,
                    rect.vertical.round() as u32,
                    rect.width.round() as u32,
                    rect.height.round() as u32,
                );

                rpass.set_pipeline(&render.render_pipeline.inner);
                rpass.set_bind_group(0, &render.bind_group, &[]);
                rpass.set_bind_group(1, texture_bind_group, &[]);

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

                // log::debug!(
                //    "rpass vertices: {:?}, indices: {}",
                //    vertex_buffer_slice,
                //    indices_count
                // );
                rpass.draw_indexed(0..indices_count as u32, 0, 0..1);
            }
        }

        encoder.finish(9000)
    }
}

impl Default for DrawTask {
    fn default() -> Self {
        Self {
            render: None,
            texture_bind_groups: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct Extension {}

impl dotrix::Extension for Extension {
    fn load(&self, manager: &dotrix::Manager) {
        manager.schedule(DrawTask::default())
    }
}
