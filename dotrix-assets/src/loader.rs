use crate::Resource;

pub enum LoadError {
    OpenFile,
    ReadFile,
}

pub trait Loader: std::any::Any + Send + 'static {
    /// Returns true if loader is able to import file of that extension
    fn can_load(&self, file_extension: &str) -> bool;
    /// Tries to load assets from file
    fn load(&self, path: &std::path::Path) -> Result<Resource, LoadError>;
}
