use crate::CHUNK_SIZE;

use std::{
    cmp::{ min, max },
};

pub const BLOCK_SIZE: usize = 4; //CHUNK_SIZE;

#[derive(Clone, Copy, Debug)]
pub struct Density<T> {
    pub values: [[[T; BLOCK_SIZE + 1]; BLOCK_SIZE + 1]; BLOCK_SIZE + 1]
}

impl Density<f32> {
    pub fn empty(value: f32) -> Self {
        Self {
            values: [[[value; BLOCK_SIZE + 1]; BLOCK_SIZE + 1]; BLOCK_SIZE + 1]
        }
    }

    pub fn pack(self) -> (Vec<i8>, bool) {
        let mut is_empty = true;
        let first_value = self.values[0][0][0];
        let packed = self.values.iter().map(
            |by_x| by_x.iter().map(
                |by_y| by_y.iter().map(
                    |&value| {
                        if is_empty && first_value * value < 0.0 {
                            is_empty = false;
                        }
                        pack_value(value)
                    }
                ).collect::<Vec<_>>()
            ).flatten().collect::<Vec<_>>()
        ).flatten().collect::<Vec<_>>();
        (packed, is_empty)
    }

    pub fn from_packed(packed: &[i8]) -> Self {
        let mut density = Self::empty(-1.0);
        let mut packed_iter = packed.into_iter();
        for by_x in density.values.iter_mut() {
            for by_y in by_x.iter_mut() {
                for value in by_y.iter_mut() {
                    *value = *(packed_iter.next()
                        .expect("unpacked values number should match the size of density map")
                    ) as f32;
                }
            }
        }
        density
    }
}

impl Density<i8> {
    fn empty() -> Self {
        Self {
            values: [[[-1; BLOCK_SIZE + 1]; BLOCK_SIZE + 1]; BLOCK_SIZE + 1]
        }
    }
}

#[inline(always)]
fn pack_value(value: f32) -> i8 {
    const LIMIT: i8 = 4;

    let packed = min(
        max(i8::MIN as i32, value.abs().ceil() as i32) * if value > 0.0 { 1 } else { -1 },
        i8::MAX as i32
    ) as i8;

    if packed > LIMIT {
        return LIMIT;
    } else if packed < -LIMIT {
        return -LIMIT;
    } else {
        return packed;
    }
}
