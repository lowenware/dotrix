use crate::CHUNK_SIZE;

use std::{
    cmp::{ min, max },
};

use dotrix_math::{ Vec3 };

/// Universal storage for density values with possibility of data compression
/// Each [`Density`] instance represents a block of density values in 3D space. It does not track
/// width, height and depth of the block, to avoid duplications of this value. It is supposed that
/// consumer will track the shape and size of the blocks.
#[derive(Clone, Debug)]
pub struct Density {
    values: Vec<i8>,
    origin_size: usize,
}

impl Density {
    /// Constructs new instance of block with Density values with or without compression.
    /// It is better to compress data for storage and keep it uncompressed for middleware
    /// calculation.
    pub fn new(values: Vec<i8>, compress: bool) -> Self {
        let origin_size = values.len();
        let mut density = Self {
            values,
            origin_size,
        };

        if compress {
            density.compress();
        }

        density
    }

    /// Constructs new instance of [`Density`] block with defined linear size from `origin_size` 
    /// and `value`
    pub fn empty(origin_size: usize, value: i8) -> Self {
        let values = vec![value; origin_size];
        let mut density = Self {
            values,
            origin_size,
        };
        density.compress();
        density
    }

    /// Returns uncompressed [`Density`] values if internal array of values is not empty
    pub fn values(&self) -> Option<Vec<i8>> {
        if self.values.len() > 0 {
            Some(
                if self.is_compressed() {
                    self.extract()
                } else {
                    self.values.clone()
                }
            )
        } else {
            None
        }
    }

    pub fn set_values(&mut self, values: Vec<i8>) {
        self.values = values;
        self.compress();
    }

    /// Checks if the [`Density`] block represents a space withut a surface.
    /// It is relatively expensive operation, but definitely cheaper than attempt of the surface
    /// construction.
    pub fn is_empty(&self) -> bool {
        let first_value = self.values[0];
        let mut skip = false;
        for &v in self.values.iter() {
            if !skip {
                let mul = first_value * v;
                if mul <= 0 {
                    return false;
                }
            }
            skip = !skip;
        }
        return true;
    }

    /// Returns origin size of the [`Density block`]
    pub fn size(&self) -> usize {
        self.origin_size
    }

    /// Checks if stored [`Density`] data was compressed
    pub fn is_compressed(&self) -> bool {
        self.values.len() < self.origin_size
    }

    /// Packs float values to take less memory
    fn pack(values: &[f32]) -> Vec<i8> {
        let first_value = values[0];
        values.iter()
            .map(|&value| pack_value(value))
            .collect::<Vec<_>>()
    }

    /// Compresses stored data. Use with care: it does not check if data was already compressed
    fn compress(&mut self) {
        let input_size = self.values.len();
        let first_byte = self.values[0];
        let mut last_byte = first_byte;
        let mut counter: u8 = 0;
        let mut output = Vec::with_capacity(input_size / 2);

        for &byte in self.values.iter() {
            if last_byte == byte && counter < 0xFF {
                counter += 1;
            } else {
                output.push(last_byte);
                output.push(counter as i8);
                counter = 1;
                last_byte = byte;

                if !(output.len() < input_size) {
                    return;
                }
            }
        }

        output.push(last_byte);

        self.values = output;
    }

    /// Extratcs stored data. Use with care: id does not check if stored data is compressed
    fn extract(&self) -> Vec<i8> {
        let mut output: Vec<i8> = Vec::with_capacity(self.origin_size);
        let mut last_byte = None;

        for &i in self.values.iter() {
            if let Some(byte) = last_byte {
                let mut counter = i as u8;
                while counter != 0 {
                    output.push(byte);
                    counter -= 1;
                }
                last_byte = None;
            } else {
                last_byte = Some(i);
            }
        }

        if let Some(byte) = last_byte {
            let mut counter = self.origin_size - output.len();
            while counter != 0 {
                output.push(byte);
                counter -= 1;
            }
        }

        output
    }

    /// Interpolates a density value inside of the [`Density`]
    ///  - `block_size` is a number of density values per cube side
    ///  - `voxel_size` is a distance in space between points where [`Density`] was measured
    ///  - `point` is a coordinate in 3d space where [`Density`] value has to be calculated
    pub fn value(&self, block_size: usize, voxel_size: usize, point: &Vec3) -> Option<f32> {
        let values = if self.is_compressed() { Some(self.extract()) } else { None };
        let density = values.as_ref().unwrap_or_else(|| &self.values);

        let x0 = Self::align_voxel_axis((point.x / voxel_size as f32).floor() as i32, block_size);
        let y0 = Self::align_voxel_axis((point.y / voxel_size as f32).floor() as i32, block_size);
        let z0 = Self::align_voxel_axis((point.z / voxel_size as f32).floor() as i32, block_size);

        let block_size_sq = block_size * block_size;
        let i000 = x0 * block_size_sq + y0 * block_size + z0;
        let i100 = i000 + block_size_sq;
        let i001 = i000 + 1;
        let i101 = i100 + 1;
        let i010 = i000 + block_size;
        let i110 = i010 + block_size_sq;
        let i011 = i010 + 1;
        let i111 = i110 + 1;

        let x0f = (x0 * voxel_size) as f32;
        let x1f = ((x0 + 1) * voxel_size) as f32;
        let y0f = (y0 * voxel_size) as f32;
        let y1f = ((y0 + 1) * voxel_size) as f32;
        let z0f = (z0 * voxel_size) as f32;
        let z1f = ((z0 + 1) * voxel_size) as f32;

        /*
        let x_00 = Self::lerp(point.x, x0f, x1f, self.density[x0][y0][z0], self.density[x1][y0][z0]);
        let x_01 = Self::lerp(point.x, x0f, x1f, self.density[x0][y0][z1], self.density[x1][y0][z1]);
        let x_10 = Self::lerp(point.x, x0f, x1f, self.density[x0][y1][z0], self.density[x1][y1][z0]);
        let x_11 = Self::lerp(point.x, x0f, x1f, self.density[x0][y1][z1], self.density[x1][y1][z1]);
        */
        let x_00 = Self::lerp(point.x, x0f, x1f, density[i000] as f32, density[i100] as f32);
        let x_01 = Self::lerp(point.x, x0f, x1f, density[i001] as f32, density[i101] as f32);
        let x_10 = Self::lerp(point.x, x0f, x1f, density[i010] as f32, density[i110] as f32);
        let x_11 = Self::lerp(point.x, x0f, x1f, density[i011] as f32, density[i111] as f32);
        let xz_0 = Self::lerp(point.z, z0f, z1f, x_00, x_01);
        let xz_1 = Self::lerp(point.z, z0f, z1f, x_10, x_11);
        Some(Self::lerp(point.y, y0f, y1f, xz_0, xz_1))
    }

    #[inline(always)]
    fn align_voxel_axis(pos: i32, block_size: usize) -> usize {
        if pos < 0 { 0 }
        else if pos < (block_size as i32 - 1) { pos as usize }
        else { block_size - 2 }
    }

    /// Linearly interpolates density values between two positions
    fn lerp(position: f32, position0: f32, position1: f32, value0: f32, value1: f32) -> f32 {
        let ratio = (position - position0) / (position1 - position0);
        let delta_value = value1 - value0;
        value0 + ratio * delta_value
    }
}

impl From<&[f32]> for Density {
    /// Constructs [`Density`] from array of float values
    fn from(values: &[f32]) -> Self {
        let values = Self::pack(values);
        Self::new(values, true)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_and_extract() {
        let mut source = (0..512).map(|v| v as f32).collect::<Vec<_>>();
        source[32] = -4.0;

        let mut density = Density::from(source.as_slice());
        assert_eq!(density.is_compressed(), true);

        let extracted = density.extract();
        assert_eq!(Density::pack(&source), extracted);
    }
}
