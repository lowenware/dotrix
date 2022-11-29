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
