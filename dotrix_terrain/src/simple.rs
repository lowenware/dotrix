/// Simple terrain generator

use dotrix_core::assets::{ Mesh, Texture };
use dotrix_core::ray::Ray;
use dotrix_math::{ InnerSpace, Vec3 };

use crate::{ Component, Generator, HeightMap, VecXZ };

#[derive(Default)]
pub struct Simple {
    pub heights: HeightMap,
    pub y_scale: f32,
    pub offset: VecXZ<i32>,
    pub dirty: bool,
}

impl Simple {
    fn world_to_map(&self, point: &Vec3, unit_size: f32) -> VecXZ<i32> {
        VecXZ::new(
            (point.x / unit_size - self.offset.x as f32).floor() as i32,
            (point.z / unit_size - self.offset.z as f32).floor() as i32,
        )
    }

    fn height(&self, map: &VecXZ<i32>) -> f32 {
        if map.x > 0 && map.z > 0 {
            self.heights.get(map.x as u32, map.z as u32).unwrap_or(0.0) * self.y_scale
        } else {
            0.0
        }
    }

    /// Returns true if point is under the terrain
    fn is_under(&self, point: &Vec3, unit_size: f32) -> bool {
        let map = self.world_to_map(&point, unit_size);
        self.height(&map) > point.y
    }

    fn binary_search_intersection(
        &self,
        depth: u32,
        min: f32,
        max: f32,
        unit_size: f32,
        ray: &Ray,
    ) -> Vec3 {
        const RECURSION_LIMIT: u32 = 200;
        const PRECISION: f32 = 0.01;
        let half = min + (max - min) / 2.0;

        let point = ray.point(half);
        let map = self.world_to_map(&point, unit_size);

        let y = self.height(&map);

        if depth == RECURSION_LIMIT || (y - point.y).abs() < PRECISION {
            return point;
        }

        let (min, max) = if self.is_under(&point, unit_size) {
            (min, half)
        } else {
            (half, max)
        };

        self.binary_search_intersection(depth + 1, min, max, unit_size, ray)
    }
}

impl Generator for Simple {
    fn get(
        &self,
        component: Component,
        position: VecXZ<i32>,
        scale: u32,
        unit_size: f32
    ) -> Option<Mesh> {
        let vertices_per_side = component.vertices_per_side();
        let units_per_side = vertices_per_side - 1;
        let half_units_per_side = units_per_side as i32 / 2;

        // TODO: calculate offset from position

        let capacity = (vertices_per_side * vertices_per_side) as usize;
        let mut positions = Vec::with_capacity(capacity);
        let mut uvs = Vec::with_capacity(capacity);
        let mut normals = vec![[0.0, 0.0, 0.0]; capacity];
        let mut indices = Vec::with_capacity(3 * 2 * units_per_side * units_per_side);

        // Map position
        let map_x = (position.x - scale as i32) * half_units_per_side;
        let map_z = (position.z - scale as i32) * half_units_per_side;

        let height_map_x0 = map_x - self.offset.x;
        let height_map_z0 = map_z - self.offset.z;

        let x0 = map_x as f32 * unit_size as f32;
        let z0 = map_z as f32 * unit_size as f32;

        for z in 0..vertices_per_side {
            let scale_z = z as i32 * scale as i32;
            let world_z = z0 + scale_z as f32 * unit_size;
            let height_map_z = height_map_z0 + scale_z;
            for x in 0..vertices_per_side {
                let scale_x = x as i32 * scale as i32;
                let world_x = x0 + scale_x as f32 * unit_size;
                let height_map_x = height_map_x0 + scale_x;
                let height = if height_map_x > 0 && height_map_z > 0 {
                    self.heights.get(
                        (height_map_x as i32) as u32,
                        (height_map_z as i32) as u32
                    )
                } else { None };

                let world_y = self.y_scale * height.unwrap_or(0.0);

                positions.push([world_x, world_y, world_z]);
                uvs.push([
                    x as f32 / units_per_side as f32,
                    z as f32 / units_per_side as f32,
                ]);
            }
        }

        for z in 0..units_per_side {
            let i = (z * vertices_per_side) as u32;
            for x in 0..units_per_side {
                let i00 = i + x as u32;
                let i10 = i00 + 1;
                let i01 = i00 + vertices_per_side as u32;
                let i11 = i01 + 1;

                indices.push(i10);
                indices.push(i00);
                indices.push(i01);
                indices.push(i10);
                indices.push(i01);
                indices.push(i11);
            }
        }
        let indices_count = indices.len();
        for i in (0..indices_count).step_by(3) {
            let i0 = indices[i] as usize;
            let i1 = indices[i + 1] as usize;
            let i2 = indices[i + 2] as usize;
            // get the face
            let p0 = Vec3::from(positions[i0]);
            let p1 = Vec3::from(positions[i1]);
            let p2 = Vec3::from(positions[i2]);

            let n1 = p1 - p0;
            let n2 = p2 - p0;
            let normal = n1.cross(n2).normalize();

            normals[i0] = (Vec3::from(normals[i0]) + normal).into();
            normals[i1] = (Vec3::from(normals[i1]) + normal).into();
            normals[i2] = (Vec3::from(normals[i2]) + normal).into();
        }

        for normal in normals.iter_mut() {
            let normalized = Vec3::from(*normal).normalize();
            normal[0] = normalized.x;
            normal[1] = normalized.y;
            normal[2] = normalized.z;
        }

        let mut mesh = Mesh::default();
        mesh.with_vertices(&positions);
        mesh.with_vertices(&normals);
        mesh.with_vertices(&uvs);
        mesh.with_indices(&indices);

        Some(mesh)
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self, value: bool) {
        self.dirty = value;
    }

    fn set_y_scale(&mut self, value: f32) {
        self.y_scale = value;
    }

    fn set_offset(&mut self, offset_x: i32, offset_z: i32) {
        self.offset.x = offset_x;
        self.offset.z = offset_z;
    }

    fn intersection(&self, ray: &Ray, range: f32, unit_size: f32) -> Option<Vec3> {
        if let Some(direction) = ray.direction.as_ref() {
            if let Some(origin) = ray.origin.as_ref() {
                let target = origin + direction * range;

                if self.is_under(&target, unit_size) && !self.is_under(&origin, unit_size) {
                    return Some(
                        self.binary_search_intersection(0, 0.0, range, unit_size, ray)
                    );
                }
            }
        }

        None
    }

    fn modify(&mut self, point: &Vec3, values: &[f32], size: u32, unit_size: f32) {
        let radius = size / 2;
        let mut map = self.world_to_map(point, unit_size);
        map.x -= radius as i32;
        map.z -= radius as i32;
        for u in 0..size {
            let index = u * size;
            let x = map.x + u as i32;
            if x > 0 {
                for v in 0..size {
                    let z = map.z + v as i32;
                    if z > 0 {
                        self.heights.add(x as u32, z as u32, values[(index + v) as usize]);
                    }
                }
            }
        }
        self.dirty = true;
    }

    fn flatten(&mut self, point: &Vec3, values: &[f32], size: u32, unit_size: f32) {
        let radius = size / 2;
        let mut map = self.world_to_map(point, unit_size);
        map.x -= radius as i32;
        map.z -= radius as i32;
        let mut target = 0.0;
        let mut cnt = 0;
        for u in 0..size {
            let index = u * size;
            let x = map.x + u as i32;
            if x > 0 {
                for v in 0..size {
                    let z = map.z + v as i32;
                    if z > 0 {
                        if let Some(value) = self.heights.get(x as u32, z as u32) {
                            target += value;
                            cnt += 1;
                        }
                    }
                }
            }
        }
        target = target / cnt as f32;
        for u in 0..size {
            let index = u * size;
            let x = map.x + u as i32;
            if x > 0 {
                for v in 0..size {
                    let z = map.z + v as i32;
                    if z > 0 {
                        if let Some(value) = self.heights.get(x as u32, z as u32) {
                            let value = (target - value) * values[(index + v) as usize];
                            self.heights.add(x as u32, z as u32, value);
                        }
                    }
                }
            }
        }

        self.dirty = true;
    }

    fn resize(&mut self, size_x: u32, size_z: u32) {
        self.heights.resize(size_x as usize, size_z as usize);
        self.dirty = true;
    }

    fn reset(&mut self) {
        self.heights.reset();
        self.dirty = true;
    }

    fn export(&self, file: &std::path::Path) {
        let texture = self.heights.texture(self.y_scale);
        if let Ok(()) = image::save_buffer_with_format(
            file,
            texture.data.as_slice(),
            texture.width,
            texture.height,
            image::ColorType::Rgba8,
            image::ImageFormat::Png
        ) {
            println!("Texture saved to {:?}", file);
        }
    }
}

impl From<HeightMap> for Simple {
    fn from(heights: HeightMap) -> Self {
        let height_map_size = heights.size();
        let offset = VecXZ::new(
            -(height_map_size.x as i32) / 2,
            -(height_map_size.z as i32) / 2
        );

        Self {
            heights,
            offset,
            y_scale: 100.0,
            dirty: true
        }
    }
}
