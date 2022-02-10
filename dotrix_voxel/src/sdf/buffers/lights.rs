use dotrix_core::{ecs::System, Application};

mod data;
mod storage;

pub use data::Light;
pub use storage::Buffer as LightStorageBuffer;

pub(super) fn extension(app: &mut Application) {
    app.add_system(System::from(storage::load));
    app.add_system(System::from(storage::startup));
}
