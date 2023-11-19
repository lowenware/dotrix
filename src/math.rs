//! Dotrix wrapper around cgmath
//!
//! This crate will either turn into our native implementation of become a wrapper to nalgebra.

#![doc(html_logo_url = "https://raw.githubusercontent.com/lowenware/dotrix/master/logo.png")]
#![warn(missing_docs)]

pub mod math;
pub use cgmath::num_traits::clamp;
pub use cgmath::num_traits::clamp_min;
pub use cgmath::perspective;
pub use cgmath::Deg;
pub use cgmath::EuclideanSpace;
pub use cgmath::InnerSpace;
pub use cgmath::MetricSpace;
pub use cgmath::PerspectiveFov;
pub use cgmath::Rad;
pub use cgmath::Rotation3;
pub use cgmath::SquareMatrix;
pub use cgmath::VectorSpace;
pub use math::slerp;

pub type Mat3 = cgmath::Matrix3<f32>;
/// 4x4 Matrix of f32
pub type Mat4 = cgmath::Matrix4<f32>;
/// 2 dimentional point of f32
pub type Point2 = cgmath::Point2<f32>;
/// 3 dimentional point of f32
pub type Point3 = cgmath::Point3<f32>;
/// 4 dimentional vector of f32
pub type Vec4 = cgmath::Vector4<f32>;
/// 3 dimentional vector of f32
pub type Vec3 = cgmath::Vector3<f32>;
/// 3 dimentional vector of i32
pub type Vec3i = cgmath::Vector3<i32>;
/// 4 dimentional vector of i32
pub type Vec4i = cgmath::Vector4<i32>;
/// 2 dimentional vector of f32
pub type Vec2 = cgmath::Vector2<f32>;
/// 2 dimentional vector of i32
pub type Vec2i = cgmath::Vector2<i32>;
/// 2 dimentional vector of u32
pub type Vec2u = cgmath::Vector2<u32>;
/// Quaternion of f32
pub type Quat = cgmath::Quaternion<f32>;

/// Interpolates quaternion
pub fn slerp(left: Quat, right: Quat, amount: f32) -> Quat {
    let num2;
    let num3;
    let num = amount;
    let mut num4 = (((left.v.x * right.v.x) + (left.v.y * right.v.y)) + (left.v.z * right.v.z))
        + (left.s * right.s);
    let mut flag = false;
    if num4 < 0.0 {
        flag = true;
        num4 = -num4;
    }
    if num4 > 0.999_999 {
        num3 = 1.0 - num;
        num2 = if flag { -num } else { num };
    } else {
        let num5 = num4.acos();
        let num6 = 1.0 / num5.sin();
        num3 = ((1.0 - num) * num5).sin() * num6;
        num2 = if flag {
            -(num * num5).sin() * num6
        } else {
            (num * num5).sin() * num6
        };
    }
    Quat::new(
        (num3 * left.s) + (num2 * right.s),
        (num3 * left.v.x) + (num2 * right.v.x),
        (num3 * left.v.y) + (num2 * right.v.y),
        (num3 * left.v.z) + (num2 * right.v.z),
    )
}
