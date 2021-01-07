pub mod math;
pub use math::slerp;
pub use cgmath::num_traits::clamp;
pub use cgmath::perspective;
pub use cgmath::VectorSpace;
pub use cgmath::InnerSpace;
pub use cgmath::MetricSpace;
pub use cgmath::Rotation3;
pub use cgmath::SquareMatrix;
pub use cgmath::Rad;
pub use cgmath::Deg;

pub type Mat4 = cgmath::Matrix4<f32>;
pub type Point3 = cgmath::Point3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec3i = cgmath::Vector3<i32>;
pub type Vec4i = cgmath::Vector4<i32>;
pub type Vec2 = cgmath::Vector2<f32>;
pub type Quat = cgmath::Quaternion<f32>;


// TODO: add real tests after removal of cgmath dependency
/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
 */
