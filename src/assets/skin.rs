
pub struct Skin {
    pub inverse_bind_matrices: Vec<cgmath::Matrix4<f32>>,
}


impl Skin {
    pub fn new(inverse_bind_matrices: Vec<cgmath::Matrix4<f32>>) -> Self {
        Self {
            inverse_bind_matrices,
        }
    }
}
