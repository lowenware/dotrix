use dotrix_core::renderer::Pipeline;

/// Shadow Pipeline component
pub struct Shadow {
    /// Shadow Pre-Rendering pipeline
    pub pipeline: Pipeline,
}

/// Lights startup system
pub fn startup(mut globals: Mut<Globals>) {
    globals.set(Shadows::default());
}
