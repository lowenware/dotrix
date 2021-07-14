use dotrix_math::{ Vec3 };

/// Voxel Map contains density values for 16x16x16 voxel chunk
#[derive(Copy, Clone)]
pub struct VoxelMap {
    /// Density values for voxels
    pub density: [[[f32; 17]; 17]; 17],
}

impl VoxelMap {
    /// Constructs new [`VoxelMap`]
    pub fn new(density: [[[f32; 17]; 17]; 17]) -> Self {
        Self {
            density,
        }
    }

    /// Interpolates a density value inside of the [`VoxelMap`]
    pub fn value(&self, voxel_size: usize, point: &Vec3) -> Option<f32> {
        let x0 = Self::align_voxel_axis((point.x / voxel_size as f32).floor() as i32);
        let y0 = Self::align_voxel_axis((point.y / voxel_size as f32).floor() as i32);
        let z0 = Self::align_voxel_axis((point.z / voxel_size as f32).floor() as i32);

        let x1 = x0 + 1;
        let y1 = y0 + 1;
        let z1 = z0 + 1;

        let x0f = (x0 * voxel_size) as f32;
        let x1f = (x1 * voxel_size) as f32;
        let y0f = (y0 * voxel_size) as f32;
        let y1f = (y1 * voxel_size) as f32;
        let z0f = (z0 * voxel_size) as f32;
        let z1f = (z1 * voxel_size) as f32;

        let x_00 = Self::lerp(point.x, x0f, x1f, self.density[x0][y0][z0], self.density[x1][y0][z0]);
        let x_01 = Self::lerp(point.x, x0f, x1f, self.density[x0][y0][z1], self.density[x1][y0][z1]);
        let x_10 = Self::lerp(point.x, x0f, x1f, self.density[x0][y1][z0], self.density[x1][y1][z0]);
        let x_11 = Self::lerp(point.x, x0f, x1f, self.density[x0][y1][z1], self.density[x1][y1][z1]);
        let xz_0 = Self::lerp(point.z, z0f, z1f, x_00, x_01);
        let xz_1 = Self::lerp(point.z, z0f, z1f, x_10, x_11);
        Some(Self::lerp(point.y, y0f, y1f, xz_0, xz_1))
    }

    #[inline(always)]
    fn align_voxel_axis(pos: i32) -> usize {
        if pos < 0 { 0 }
        else if pos < 16 { pos as usize }
        else { 15 }
    }

    /// Linearly interpolates density values between two positions
    pub fn lerp(position: f32, position0: f32, position1: f32, value0: f32, value1: f32) -> f32 {
        let ratio = (position - position0) / (position1 - position0);
        let delta_value = value1 - value0;
        value0 + ratio * delta_value
    }
}
