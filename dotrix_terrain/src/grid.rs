use std::collections::HashMap;

use dotrix_math::{
    Vec3i
};

use crate::Density;


/// Grid is a container for Density values splitted in equally sized Blocks.
pub struct Grid {
    blocks: HashMap<Vec3i, Density>,
    /// Number of density values per side
    block_size: usize,
}

impl Grid {
    /// Constructs new Grid instance
    pub fn new(block_size: usize) -> Self {
        Self {
            blocks: HashMap::new(),
            block_size
        }
    }

    /// Constructs Grid with a flat density map of specified `size`
    pub fn flat(size: usize, block_size: usize) -> Self {
        let mut grid = Self::new(block_size);

        let blocks = ((size as f32 / block_size as f32) / 2.0).ceil() as i32;
        let values_in_block = block_size * block_size * block_size;

        // prepare flat density block
        let above = Density::empty(values_in_block, -1);
        let bellow = Density::empty(values_in_block, 1);
        let mut values = bellow.values().expect("Density values to be set");

        for x in 0..block_size {
            let xx = x * block_size * block_size + (block_size - 1) * block_size;
            for z in 0..block_size {
                values[xx + z] = 0;
            }
        }
        let surface = Density::new(values, true);

        for x in -blocks..blocks {
            for y in -blocks..blocks {
                for z in -blocks..blocks {
                    let index = Vec3i::new(
                        x * block_size as i32,
                        y * (block_size as i32),
                        z * block_size as i32
                    );
                    let block = if y < -1 {
                        bellow.clone()
                    } else if y > -1 {
                        above.clone()
                    } else {
                        surface.clone()
                    };
                    grid.blocks.insert(index, block);
                }
            }
        }
        grid
    }

    /// Gets a chunk of density values of requested size at requested base
    pub fn load(&self, from: Vec3i, size: usize) -> Density {
        let size_sq = size * size;
        let mut values = vec![-1; size_sq * size];

        let to = Vec3i::new(from.x + size as i32 - 1, from.y + size as i32 - 1, from.z + size as i32 - 1);

        let block_size = self.block_size;
        let block_from = Self::get_block_index(&from, block_size);
        let block_to = Self::get_block_index(&to, block_size);

        let block_size_sq = block_size * block_size;

        for block_x in (block_from.x..=block_to.x).step_by(block_size) {
            for block_y in (block_from.y..=block_to.y).step_by(block_size) {
                for block_z in (block_from.z..=block_to.z).step_by(block_size) {
                    let block_index = Vec3i::new(block_x, block_y, block_z);
                    if let Some(block) = self.blocks.get(&block_index) {
                        if let Some(density) = block.values() {
                            let src_x = Self::get_in_block_offset(from.x, block_x, block_size);
                            let src_y = Self::get_in_block_offset(from.y, block_y, block_size);
                            let src_z = Self::get_in_block_offset(from.z, block_z, block_size);
                            let to_x = Self::get_in_block_offset(to.x, block_x, block_size - 1) + 1;
                            let to_y = Self::get_in_block_offset(to.y, block_y, block_size - 1) + 1;
                            let to_z = Self::get_in_block_offset(to.z, block_z, block_size - 1) + 1;

                            for x in src_x .. to_x {
                                let xx = block_size_sq * x;
                                for y in src_y .. to_y {
                                    let xy = xx + block_size * y;
                                    for z in src_z .. to_z {
                                        let xyz = xy + z;
                                        let i = size_sq as i32 * (block_x + x as i32 - from.x)
                                            + size as i32 * (block_y + y as i32 - from.y)
                                            + (block_z + z as i32 - from.z);
                                        values[i as usize] = density[xyz];
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Density::new(values, false)
    }

    /// Stores a chunk of density values in the Grid
    pub fn save(&mut self, from: Vec3i, size: usize, values: Vec<i8>) {
        let size_sq = size * size;

        let to = Vec3i::new(from.x + size as i32 - 1, from.y + size as i32 - 1, from.z + size as i32 - 1);

        let block_size = self.block_size;
        let block_from = Self::get_block_index(&from, block_size);
        let block_to = Self::get_block_index(&to, block_size);

        let block_size_sq = block_size * block_size;


        for block_x in (block_from.x..=block_to.x).step_by(block_size) {
            for block_y in (block_from.y..=block_to.y).step_by(block_size) {
                for block_z in (block_from.z..=block_to.z).step_by(block_size) {
                    let block_index = Vec3i::new(block_x, block_y, block_z);
                    if let Some(mut block) = self.blocks.get_mut(&block_index) {
                        if let Some(mut density) = block.values() {
                            let src_x = Self::get_in_block_offset(from.x, block_x, block_size);
                            let src_y = Self::get_in_block_offset(from.y, block_y, block_size);
                            let src_z = Self::get_in_block_offset(from.z, block_z, block_size);
                            let to_x = Self::get_in_block_offset(to.x, block_x, block_size - 1) + 1;
                            let to_y = Self::get_in_block_offset(to.y, block_y, block_size - 1) + 1;
                            let to_z = Self::get_in_block_offset(to.z, block_z, block_size - 1) + 1;

                            for x in src_x .. to_x {
                                let xx = block_size_sq * x;
                                for y in src_y .. to_y {
                                    let xy = xx + block_size * y;
                                    for z in src_z .. to_z {
                                        let xyz = xy + z;
                                        let i = size_sq as i32 * (block_x + x as i32 - from.x)
                                            + size as i32 * (block_y + y as i32 - from.y)
                                            + (block_z + z as i32 - from.z);
                                        density[xyz] = values[i as usize];
                                    }
                                }
                            }
                            block.set_values(density);
                        }
                    }
                }
            }
        }
    }

    fn get_block_index(base: &Vec3i, block_size: usize) -> Vec3i {
        let block_size = block_size as i32;
        let x = block_size * (base.x as f32 / block_size as f32).floor() as i32;
        let y = block_size * (base.y as f32 / block_size as f32).floor() as i32;
        let z = block_size * (base.z as f32 / block_size as f32).floor() as i32;
        Vec3i::new(x, y, z)
    }

    fn get_in_block_offset(from: i32, block: i32, size: usize) -> usize {
        let offset = from - block;
        if offset < 0 {
            0
        } else if offset > size as i32 {
            size
        } else {
            offset as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_block_indices() {
        assert_eq!(Grid::get_block_index(&Vec3i::new(-1, -1, -1), 2), Vec3i::new(-2, -2, -2));
        assert_eq!(Grid::get_block_index(&Vec3i::new(0, -1, -1), 2), Vec3i::new(0, -2, -2));
        assert_eq!(Grid::get_block_index(&Vec3i::new(0, 0, 0), 2), Vec3i::new(0, 0, 0));
        assert_eq!(Grid::get_block_index(&Vec3i::new(-1, 0, 0), 2), Vec3i::new(-2, 0, 0));

        let block = Vec3i::new(-4, 0, -2);
        assert_eq!(Grid::get_block_index(&Vec3i::new(-4, 0, -2), 2), block);
        assert_eq!(Grid::get_block_index(&Vec3i::new(-3, 0, -2), 2), block);
        assert_eq!(Grid::get_block_index(&Vec3i::new(-3, 0, -1), 2), block);
        assert_eq!(Grid::get_block_index(&Vec3i::new(-4, 0, -1), 2), block);

        let block = Vec3i::new(3, 0, -3);
        assert_eq!(Grid::get_block_index(&Vec3i::new(3, 0, -3), 3), block);
        assert_eq!(Grid::get_block_index(&Vec3i::new(5, 0, -3), 3), block);
        assert_eq!(Grid::get_block_index(&Vec3i::new(3, 0, -1), 3), block);
        assert_eq!(Grid::get_block_index(&Vec3i::new(5, 0, -1), 3), block);
    }

    #[test]
    fn load_values_from_grid() {
        let block_size = 2;
        let mut grid = Grid::new(block_size);
        let mut values = vec![0; block_size * block_size * block_size];
        let mut counter = 1;
        for x in 0..block_size {
            let xx = block_size * block_size * x;
            for y in 0..block_size {
                let xy = xx + block_size * y;
                for z in 0..block_size {
                    let xyz = xy + z;
                    values[xyz] = counter;
                    counter += 1;
                }
            }
        }
        let shift = block_size as i32;
        grid.blocks.insert(Vec3i::new(0, 0, 0), Density::new(values.clone(), false));
        grid.blocks.insert(Vec3i::new(-shift, 0, 0), Density::new(values.clone(), false));
        grid.blocks.insert(Vec3i::new(0, 0, -shift), Density::new(values.clone(), false));
        grid.blocks.insert(Vec3i::new(-shift, 0, -shift), Density::new(values.clone(), false));
        grid.blocks.insert(Vec3i::new(0, -shift, 0), Density::new(values.clone(), false));
        grid.blocks.insert(Vec3i::new(-shift, -shift, 0), Density::new(values.clone(), false));
        grid.blocks.insert(Vec3i::new(0, -shift, -shift), Density::new(values.clone(), false));
        grid.blocks.insert(Vec3i::new(-shift, -shift, -shift), Density::new(values, false));

        let select = grid.load(Vec3i::new(-1, -1, -1), 2)
            .values()
            .expect("Must have values");
        let result = vec![8, 7, 6, 5, 4, 3, 2, 1];

        assert_eq!(select, result);
    }
}
