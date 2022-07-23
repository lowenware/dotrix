use crate::Asset;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LoadError {
    NotSupported,
    OnOpen,
    OnParse(String),
}

pub trait Loader: std::any::Any + Send + 'static {
    /// Returns true if loader is able to import file of that extension
    fn can_load(&self, path: &std::path::Path) -> bool;
    /// Tries to load assets from file buffer
    /// Returns pair of namespace and imported asset
    fn load(&self, path: &std::path::Path, data: Vec<u8>) -> Vec<Box<dyn Asset>>;
}
