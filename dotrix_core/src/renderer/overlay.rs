use crate::{
    ecs::{ Const, Mut },
    renderer::{ Widget },
    services::{
        Assets,
        Input,
        Renderer,
    },
};

use std::any::Any;

pub struct Overlay {
    pub provider: Box<dyn Provider>,
}

impl Overlay {
    pub fn new(provider: Box<dyn Provider>) -> Self {
        Self {
            provider
        }
    }

    pub fn provider<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.provider.downcast_ref::<T>()
    }

    pub fn provider_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.provider.downcast_mut::<T>()
    }

    pub fn update(
        &mut self, 
        assets: &mut Assets,
        input: &Input,
        scale_factor: f32,
        surface_width: f32,
        surface_height: f32,
    ) {
        self.provider.feed(assets, input, scale_factor, surface_width, surface_height);
    }

    pub fn widgets(
        &self,
        scale_factor: f32,
        surface_width: f32,
        surface_height: f32,
    ) -> Vec<Widget> {
        self.provider.tessellate(scale_factor, surface_width, surface_height)
    }
}

pub trait Provider: Any + Send + Sync {
    fn feed(
        &mut self,
        assets: &mut Assets,
        input: &Input,
        scale_factor: f32,
        surface_width: f32,
        surface_height: f32,
    );

    fn tessellate(
        &self,
        scale_factor: f32,
        surface_width: f32,
        surface_height: f32,
    ) -> Vec<Widget>;

}

impl dyn Provider {
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&*(self as *const dyn Provider as *const T)) }
        } else {
            None
        }
    }

    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&mut *(self as *mut dyn Provider as *mut T)) }
        } else {
            None
        }
    }

    #[inline]
    fn is<T: Any>(&self) -> bool {
        std::any::TypeId::of::<T>() == self.type_id()
    }
}

pub fn overlay_update(
    mut assets: Mut<Assets>,
    input: Const<Input>,
    mut renderer: Mut<Renderer>
) {
    let (width, height) = renderer.display_size();
    let scale_factor = renderer.scale_factor();
    for overlay in &mut renderer.overlay {
        overlay.update(&mut assets, &input, scale_factor, width as f32, height as f32);
    }
}
