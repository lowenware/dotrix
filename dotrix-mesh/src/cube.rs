use crate::Mesh;

/// Cube primitive
pub struct Cube {
    /// Cube size
    pub size: f32,
    /// Texture UVs
    pub tex_uvs: Vec<[f32; 2]>,
    /// Indices
    pub indices: Vec<u32>,
}

impl Cube {
    /// Constructs new Cube instance
    pub fn new(size: f32) -> Self {
        Self {
            size,
            tex_uvs: vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [0.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [1.0, 0.0],
                [1.0, 0.0],
                [0.0, 1.0],
                [0.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [1.0, 0.0],
                [0.0, 0.0],
                [1.0, 1.0],
                [0.0, 0.0],
                [1.0, 1.0],
                [1.0, 0.0],
                [0.0, 1.0],
                [0.0, 1.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 0.0],
            ],
            indices: vec![
                0, 2, 1, 0, 3, 2, // front
                4, 5, 6, 4, 7, 5, // right
                8, 9, 10, 8, 11, 9, // back
                12, 13, 14, 12, 15, 13, // left
                16, 17, 18, 16, 19, 17, // top
                20, 21, 22, 20, 23, 21, // bottom
            ],
        }
    }

    /// Returns vertices positions of the cube
    pub fn positions(&self) -> Vec<[f32; 3]> {
        let half_width = self.size / 2.0;
        vec![
            [-half_width, -half_width, -half_width], // 0 -> 0
            [half_width, -half_width, -half_width],  // 1 -> 1
            [half_width, half_width, -half_width],   // 2 -> 2
            [-half_width, half_width, -half_width],  // 3 -> 3
            [half_width, -half_width, -half_width],  // 1 -> 4
            [half_width, half_width, half_width],    // 6 -> 5
            [half_width, -half_width, half_width],   // 5 -> 6
            [half_width, half_width, -half_width],   // 2 -> 7
            [half_width, -half_width, half_width],   // 5 -> 8
            [-half_width, half_width, half_width],   // 7 -> 9
            [-half_width, -half_width, half_width],  // 4 -> 10
            [half_width, half_width, half_width],    // 6 -> 11
            [-half_width, -half_width, half_width],  // 4 -> 12
            [-half_width, half_width, -half_width],  // 3 -> 13
            [-half_width, -half_width, -half_width], // 0 -> 14
            [-half_width, half_width, half_width],   // 7 -> 15
            [-half_width, half_width, -half_width],  // 3 -> 16
            [half_width, half_width, half_width],    // 6 -> 17
            [half_width, half_width, -half_width],   // 2 -> 18
            [-half_width, half_width, half_width],   // 7 -> 19
            [-half_width, -half_width, half_width],  // 4 -> 20
            [half_width, -half_width, -half_width],  // 1 -> 21
            [half_width, -half_width, half_width],   // 5 -> 22
            [-half_width, -half_width, -half_width], // 0 -> 23
        ]
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new(1.0)
    }
}
