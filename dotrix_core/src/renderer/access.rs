/// Defines the generic access modes
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Access {
    /// Read only access
    ReadOnly,
    /// Write only access
    WriteOnly,
    /// Read and Write access
    ReadWrite,
}

impl From<&Access> for wgpu::StorageTextureAccess {
    fn from(obj: &Access) -> Self {
        match obj {
            Access::ReadOnly => Self::ReadOnly,
            Access::WriteOnly => Self::WriteOnly,
            Access::ReadWrite => Self::ReadWrite,
        }
    }
}
