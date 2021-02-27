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

/// Wrapper for overlay [`Provider`] interface
///
/// This structure is being constructed automatically inside of the
/// [`crate::services::Renderer::add_overlay`] method.
pub struct Overlay {
    /// Boxed overlay provider
    pub provider: Box<dyn Provider>,
}

impl Overlay {
    /// Constructs new wrapper
    pub fn new(provider: Box<dyn Provider>) -> Self {
        Self {
            provider
        }
    }

    /// Casts the [`Provider`] reference down by its type
    pub fn provider<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.provider.downcast_ref::<T>()
    }

    /// Casts the [`Provider`] mutable reference down by its type
    pub fn provider_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.provider.downcast_mut::<T>()
    }

    /// Calls [`Provider::feed`] method
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

    /// Calls [`Provider::tessellate`] method
    pub fn widgets(
        &self,
        scale_factor: f32,
        surface_width: f32,
        surface_height: f32,
    ) -> Vec<Widget> {
        self.provider.tessellate(scale_factor, surface_width, surface_height)
    }
}

/// Overlay interface
///
/// To implement custom overlay, it is necessary to define to methods of the trait and add it to
/// to the [`crate::services::Renderer`] service
///
/// ## Example
/// ```
/// use dotrix_core::{
///     ecs::Mut,
///     renderer::{ OverlayProvider, Widget },
///     services::{Assets, Input, Renderer},
/// };
///
/// struct MyOverlay {
///     // properties
/// }
///
/// impl OverlayProvider for MyOverlay {
///
///     fn feed(
///         &mut self,
///         assets: &mut Assets,
///         input: &Input,
///         scale_factor: f32,
///         surface_width: f32,
///         surface_height: f32,
///     ) {
///         // handle inputs
///     }
///
///     fn tessellate(
///         &self,
///         scale_factor: f32,
///         surface_width: f32,
///         surface_height: f32,
///     ) -> Vec<Widget> {
///         let widgets = Vec::new();
///         // populate widgets
///         widgets
///     }
/// }
///
/// fn startup(mut renderer: Mut<Renderer>) {
///     renderer.add_overlay(Box::new(MyOverlay {}));
/// }
/// ```
pub trait Provider: Any + Send + Sync {
    /// Feeds the [`Provider`] with inputs
    fn feed(
        &mut self,
        assets: &mut Assets,
        input: &Input,
        scale_factor: f32,
        surface_width: f32,
        surface_height: f32,
    );

    /// Returns tesselated widgets for current frame
    fn tessellate(
        &self,
        scale_factor: f32,
        surface_width: f32,
        surface_height: f32,
    ) -> Vec<Widget>;

}

impl dyn Provider {
    /// Casts down the reference
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

    /// Casts down the mutual reference
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

    /// Checks if the reference is of specific type
    #[inline]
    fn is<T: Any>(&self) -> bool {
        std::any::TypeId::of::<T>() == self.type_id()
    }
}

/// System feeding overlays with inputs
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
