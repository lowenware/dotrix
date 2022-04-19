/// An individual voxel
#[derive(Default, Clone, Copy, Debug)]
pub struct Voxel {
    /// Voxel density value
    pub value: u8,
    /// Voxel material id
    pub material: u8,
}
