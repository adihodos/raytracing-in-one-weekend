use super::mat4::Mat4F32;
use std::convert::From;

#[rustfmt::skip]
pub fn orthographic(left: f32, top: f32, right: f32, bottom: f32, near: f32, far: f32) -> Mat4F32 {
    let width = right - left;
    let height = top - bottom;

    Mat4F32::from(
    [
        2_f32 / width,
        0_f32,
        0_f32,
        -(right + left) / width,

        0_f32,
        2_f32 / height,
        0_f32,
        -(top + bottom) / height,

        0_f32,
        0_f32,
        -2_f32 / (far - near),
        -(far + near) / (far - near),

        0_f32,
        0_f32,
        0_f32,
        1_f32,
    ])
}

#[rustfmt::skip]
pub fn orthographic_symmetric(right: f32, top: f32, near: f32, far: f32) -> Mat4F32 {
    Mat4F32::from(
    [
        1_f32 / right,
        0_f32,
        0_f32,
        0_f32,

        0_f32,
        1_f32 / top,
        0_f32,
        0_f32,

        0_f32,
        0_f32,
        -2_f32 / (far - near),
        -(far + near) / (far - near),
        
        0_f32,
        0_f32,
        0_f32,
        1_f32,
    ])
}
