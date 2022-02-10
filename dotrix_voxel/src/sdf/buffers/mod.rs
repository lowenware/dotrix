use dotrix_core::Application;

mod camera;
mod lights;
mod sdf;

pub use camera::Buffer as CameraBuffer;
pub use lights::*;
pub use sdf::SdfBufferData;

pub(super) fn extension(app: &mut Application) {
    camera::extension(app);
    lights::extension(app);
}
