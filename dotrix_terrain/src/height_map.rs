use rand::Rng;
use std::f32::consts::PI;
use dotrix_core::assets::Texture;

pub struct HeightMap {
    size: usize,
    heights: Vec<f32>,
}

impl HeightMap {

    pub fn new(size: usize) -> Self {
        let len = size * size;
        let heights = vec![0.0; len];
        Self {
            size,
            heights,
        }
    }

    pub fn from_texture(texture: &Texture, y_scale: f32) -> Self {
        let size = if texture.width == texture.height {
            texture.width as usize
        } else {
            panic!("Can't handle non-square heightmap");
        };

        let len = (texture.width * texture.height) as usize;
        let mut heights = Vec::with_capacity(len);
        let bytes_per_pixel = texture.data.len() / len;
        for i in 0..len {
            let offset = bytes_per_pixel * i;
            let mut value = 0;
            for b in 0..2 {
                value |= texture.data[offset + b] << (8 * b);
            }
            let height = y_scale * value as f32;
            heights.push(height);
        }
        println!("HeightMap: bytes per pixel = {}, size = {}, heights = {}",
            bytes_per_pixel, size, heights.len());

        Self {
            size,
            heights,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn pick<T>(&self, x: T, z: T) -> f32
    where Self: Picker<T>
    {
        self.pick_height(x, z)
    }

    pub fn add_hill(&mut self, hill_radius: f32) {
        let mut rng = rand::thread_rng();

        let angle = rng.gen_range(0.0..2.0 * PI);
        let half_size = (self.size / 2) as f32;
        let distance = rng.gen_range((hill_radius / 2.0)..(half_size - hill_radius));
        let x = half_size + angle.cos() * distance;
        let z = half_size + angle.sin() * distance;
        let hill_radius_square = hill_radius * hill_radius;

        let mut x_min = (x - hill_radius - 1.0) as usize;
        let mut x_max = (x + hill_radius + 1.0) as usize;
        let mut z_min = (z - hill_radius - 1.0) as usize;
        let mut z_max = (z + hill_radius + 1.0) as usize;

        if x_max >= self.size {
            x_max = self.size - 1;
        }

        if z_max >= self.size {
            z_max = self.size - 1;
        }

        if x_min > x_max {
            x_min = 0;
        }

        if z_min > z_max {
            z_min = 0;
        }

        for xi in x_min..x_max {
            for zi in z_min..z_max {
                let dx = x - xi as f32;
                let dz = z - zi as f32;
                let height = hill_radius_square - (dx * dx + dz * dz);
                if height > 0.0 {
                    let value = self.heights[zi * self.size + xi];
                    if height > value {
                        self.heights[zi * self.size + xi] = height;
                    }
                }
            }
        }
    }

    pub fn generate(&mut self, hills: usize, max_radius: f32) {
        let mut rng = rand::thread_rng();
        for _ in 0..hills {
            let hill_radius = rng.gen_range(1.0..max_radius);
            self.add_hill(hill_radius);
        }
        self.normalize();
    }

    pub fn normalize(&mut self) {
        let mut max = 0.0;

        for x in 0..self.size {
            for z in 0..self.size {
                let height = self.pick(x, z);
                if height > max {
                    max = height;
                }
            }
        }

        if max > 0.0 {
            for x in 0..self.size {
                for z in 0..self.size {
                    let index = z * self.size + x;
                    let height = self.heights[index];
                    self.heights[index] = 50.0 * height / max;
                }
            }
        }
    }
}

pub trait Picker<T> {
    fn pick_height(&self, x: T, z: T) -> f32;
}

impl Picker<usize> for HeightMap {
    fn pick_height(&self, x: usize, z: usize) -> f32 {
        self.heights[z * self.size + x]
    }
}

impl Picker<f32> for HeightMap {
    fn pick_height(&self, x: f32, z: f32) -> f32
    where Self: Picker<usize>
    {
        self.pick_height(z.floor() as usize, x.floor() as usize)
    }
}
