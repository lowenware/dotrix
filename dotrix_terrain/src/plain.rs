use crate::height_map::HeightMap;

pub struct Plain {
    pub positions: Vec<[f32; 3]>,
}

impl Plain {
    pub fn new(size: usize) -> Self {
        let mut positions = Vec::with_capacity(3 * 2 * size * size);
        for z in 0..size {
            let z0 = z as f32;
            let z1 = z0 + 1.0;
            for x in 0..size {
                let x0 = x as f32;
                let x1 = x0 + 1.0;
                positions.push([x0, 0.0, z0]);
                positions.push([x0, 0.0, z1]);
                positions.push([x1, 0.0, z0]);
                positions.push([x1, 0.0, z0]);
                positions.push([x0, 0.0, z1]);
                positions.push([x1, 0.0, z1]);
            }
        }
        Self {
            positions
        }
    }

    pub fn from_height_map(height_map: &HeightMap) -> Self {
        let size = height_map.size() - 1;
        let mut positions = Vec::with_capacity(3 * 2 * size * size);
        for z in 0..size {
            let z0 = z as f32;
            let z1 = z0 + 1.0;
            for x in 0..size {
                let x0 = x as f32;
                let x1 = x0 + 1.0;
                let y00 = height_map.pick(x0, z0);
                let y01 = height_map.pick(x0, z1);
                let y10 = height_map.pick(x1, z0);
                let y11 = height_map.pick(x1, z1);
                positions.push([x0, y00, z0]);
                positions.push([x0, y01, z1]);
                positions.push([x1, y10, z0]);
                positions.push([x1, y10, z0]);
                positions.push([x0, y01, z1]);
                positions.push([x1, y11, z1]);
            }
        }
        Self {
            positions
        }
    }
}
