mod widget;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use dotrix_core::assets::Shader;
use dotrix_core::ecs::{Const, Mut, Priority, System};
use dotrix_core::renderer::{
    BindGroup, Binding, Buffer, DepthBufferMode, Pipeline, PipelineLayout, RenderOptions, Sampler,
    Stage,
};
use dotrix_core::{Application, Assets, Globals, Input, Renderer, Window};

const PIPELINE_LABEL: &str = "dotrix::overlay";

pub use widget::Widget;

/// Overlay providers container
#[derive(Default)]
pub struct Overlay {
    providers: HashMap<TypeId, Box<dyn Ui>>,
    uniform: Option<Buffer>,
}

unsafe impl Sync for Overlay {}
unsafe impl Send for Overlay {}

pub struct Providers<'a> {
    iter: std::collections::hash_map::ValuesMut<'a, TypeId, Box<dyn Ui>>,
}

impl<'a> Iterator for Providers<'a> {
    type Item = &'a mut Box<dyn Ui>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl Overlay {
    pub fn providers(&mut self) -> Providers {
        Providers {
            iter: self.providers.values_mut(),
        }
    }

    pub fn set<T>(&mut self, ui: T)
    where
        T: Ui,
    {
        let type_id = TypeId::of::<T>();
        self.providers.insert(type_id, Box::new(ui));
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        let type_id = TypeId::of::<T>();
        self.providers
            .get(&type_id)
            .map(|boxed| boxed.downcast_ref::<T>().unwrap())
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Any,
    {
        let type_id = TypeId::of::<T>();
        self.providers
            .get_mut(&type_id)
            .map(|boxed| boxed.downcast_mut::<T>().unwrap())
    }

    pub fn remove<T>(&mut self)
    where
        T: Any,
    {
        let type_id = TypeId::of::<T>();
        self.providers.remove(&type_id);
    }
}

pub trait Ui: Any + Send + Sync {
    /// Feeds the [`Ui`] with inputs
    fn bind(&mut self, assets: &mut Assets, input: &mut Input, window: &Window);

    /// Returns tessellated widgets for current frame
    fn tessellate(&mut self, window: &Window) -> &mut [(Widget, Pipeline)];
}

impl dyn Ui {
    /// Casts down the reference
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&*(self as *const dyn Ui as *const T)) }
        } else {
            None
        }
    }

    /// Casts down the mutual reference
    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&mut *(self as *mut dyn Ui as *mut T)) }
        } else {
            None
        }
    }

    /// Checks if the reference is of specific type
    #[inline]
    fn is<T: Any>(&self) -> bool {
        std::any::TypeId::of::<T>() == self.type_id()
    }
}

pub fn startup(mut assets: Mut<Assets>, renderer: Const<Renderer>) {
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: String::from(include_str!("shaders/overlay.wgsl")),
        ..Default::default()
    };
    shader.load(&renderer);
    assets.store_as(shader, PIPELINE_LABEL);
}

pub fn bind(
    mut overlay: Mut<Overlay>,
    mut assets: Mut<Assets>,
    mut input: Mut<Input>,
    window: Const<Window>,
) {
    for provider in overlay.providers() {
        provider.bind(&mut assets, &mut input, &window);
    }
}

pub fn render(
    mut overlay: Mut<Overlay>,
    mut renderer: Mut<Renderer>,
    mut assets: Mut<Assets>,
    globals: Const<Globals>,
    window: Const<Window>,
) {
    let mut overlay_uniform = overlay
        .uniform
        .take()
        .unwrap_or_else(|| Buffer::uniform("Overlay Buffer"));

    let window_size = window.inner_size();
    let scale_factor = window.scale_factor();

    renderer.load_buffer(
        &mut overlay_uniform,
        bytemuck::cast_slice(&[Uniform {
            window_size: [
                window_size.x as f32 / scale_factor,
                window_size.y as f32 / scale_factor,
            ],
        }]),
    );

    for provider in overlay.providers() {
        for (widget, pipeline) in provider.tessellate(&window) {
            if pipeline.shader.is_null() {
                pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
            }

            // check if model is disabled or already rendered
            if !pipeline.cycle(&renderer) {
                continue;
            }

            widget.mesh.load(&renderer);

            if let Some(texture) = assets.get_mut(widget.texture) {
                texture.load(&renderer);
            } else {
                continue;
            }

            if !pipeline.ready(&renderer) {
                if let Some(shader) = assets.get(pipeline.shader) {
                    let sampler = globals
                        .get::<Sampler>()
                        .expect("Sampler buffer must be loaded");

                    let texture = assets.get(widget.texture).expect("Texture must be loaded");

                    renderer.bind(
                        pipeline,
                        PipelineLayout::Render {
                            label: String::from(PIPELINE_LABEL),
                            mesh: &widget.mesh,
                            shader,
                            bindings: &[
                                BindGroup::new(
                                    "Globals",
                                    vec![Binding::Uniform(
                                        "Overlay",
                                        Stage::Vertex,
                                        &overlay_uniform,
                                    )],
                                ),
                                BindGroup::new(
                                    "Locals",
                                    vec![
                                        Binding::Texture(
                                            "Texture",
                                            Stage::Fragment,
                                            &texture.buffer,
                                        ),
                                        Binding::Sampler("Sampler", Stage::Fragment, sampler),
                                    ],
                                ),
                            ],
                            options: RenderOptions {
                                depth_buffer_mode: DepthBufferMode::Disabled,
                                disable_cull_mode: true,
                                ..Default::default()
                            },
                        },
                    );
                }
            }
            renderer.draw(pipeline, &widget.mesh, &widget.draw_args);
        }
    }

    overlay.uniform = Some(overlay_uniform);
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct Uniform {
    window_size: [f32; 2],
}

unsafe impl bytemuck::Zeroable for Uniform {}
unsafe impl bytemuck::Pod for Uniform {}

pub fn extension(application: &mut Application) {
    application.add_system(System::from(startup));
    application.add_system(System::from(bind));
    application.add_system(System::from(render).with(Priority::Custom(0)));
    application.add_service(Overlay::default());
}
