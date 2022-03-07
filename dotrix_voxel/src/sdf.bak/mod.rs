use dotrix_core::Application;

mod buffers;
mod compute;
mod data;
mod render;

pub use buffers::*;
pub use data::{ComputeSdf, RenderSdf};

pub fn extension(app: &mut Application) {
    buffers::extension(app);
    render::extension(app);
    compute::extension(app);
}
