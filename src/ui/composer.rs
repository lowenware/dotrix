use dotrix_gpu as gpu;
use dotrix_input::Input;
use dotrix_types::{Frame, Id};

use crate::overlay;
use crate::Rect;

pub struct Composer<'c, 'i, 'f> {
    pub ctx: &'c mut Context,
    pub input: &'i Input,
    pub frame: &'f Frame,
    pub build_stack: Vec<overlay::Builder>,
    pub widgets: Vec<overlay::Widget>,
}

pub trait Compose: Send {
    fn compose<'c, 'i, 'f>(&self, rect: &Rect, composer: &mut Composer<'c, 'i, 'f>);
}

impl<'c, 'i, 'f> Composer<'c, 'i, 'f> {
    pub fn new(ctx: &'c mut Context, input: &'i Input, frame: &'f Frame) -> Self {
        Self {
            ctx,
            input,
            frame,
            build_stack: vec![],
            widgets: vec![],
        }
    }

    pub fn builder<'a>(
        &'a mut self,
        rect: Rect,
        texture: Id<gpu::TextureView>,
    ) -> (&'a mut overlay::Builder, bool) {
        let exists = self
            .build_stack
            .last()
            .and_then(|builder| {
                if builder.texture == texture {
                    Some(builder)
                } else {
                    None
                }
            })
            .is_some();

        //let exists = false;

        if !exists {
            self.build_stack.push(overlay::Builder::new(rect, texture));
        }
        (self.build_stack.last_mut().unwrap(), !exists)
    }

    pub fn builder_pop(&mut self) -> Option<overlay::Builder> {
        self.build_stack.pop()
    }

    pub fn add_widget(&mut self, widget: overlay::Widget) {
        self.widgets.push(widget);
    }
}
