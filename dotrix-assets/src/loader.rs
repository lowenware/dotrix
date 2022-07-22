use crate::Asset;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LoadError {
    NotSupported,
    OnOpen,
    OnParse(String),
}

pub trait Loader: std::any::Any + Send + 'static {
    /// Returns true if loader is able to import file of that extension
    fn can_load(&self, file_extension: &str) -> bool;
    /// Tries to load assets from file buffer
    /// Returns pair of namespace and imported asset
    fn load(&self, name: &str, extension: &str, data: Vec<u8>) -> Vec<Box<dyn Asset>>;
}
