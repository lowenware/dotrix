//! Dotrix wrapper around glam

pub use glam::{
    IVec2 as Vec2i, IVec3 as Vec3i, IVec4 as Vec4i, Mat3, Mat4, Quat, UVec2 as Vec2u,
    UVec3 as Vec3u, UVec4 as Vec4u, Vec2, Vec3, Vec4,
};

/*
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
*/
