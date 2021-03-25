
pub struct Voxel {
    values: [f32; 8],
}

impl Voxel {
    pub fn new(values: [f32; 8]) -> Self {
        Self {
            values
        }
    }

    pub fn case(&self) -> usize {
        let mut case_index = 0;
        for i in 0..8 {
            if self.values[i] < 0.0 {
                case_index |= 1 << i;
            }
        }
        case_index
    }
}

