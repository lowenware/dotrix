#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct SdfBufferData {
    // This transform scales the 1x1x1 cube so that it totally encloses the
    // voxels
    pub cube_transform: [[f32; 4]; 4],
    // World transform of the voxel grid
    pub world_transform: [[f32; 4]; 4],
    // Dimensions of the voxel
    pub grid_dimensions: [f32; 3],
    pub padding: [f32; 1],
}

unsafe impl bytemuck::Zeroable for SdfBufferData {}
unsafe impl bytemuck::Pod for SdfBufferData {}
