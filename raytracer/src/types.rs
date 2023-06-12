#![allow(dead_code)]

#[cfg(feature = "fp_double_precision")]
mod rtow_types {
    pub type Real = f64;
    pub const C_255_999: Real = 255.999f64;
    pub const C_256: Real = 256f64;
    pub const C_INFINITY: Real = std::f64::INFINITY;
    pub const C_PI: Real = std::f64::consts::PI;
    pub const C_ONE: Real = 1.0;
    pub const C_HALF_ONE: Real = 0.5;
    pub const C_TWO: Real = 2.0;
    pub const C_ZERO: Real = 0.0;
    pub const C_TWO_PI: Real = C_PI * C_TWO;
    pub const FP_MODEL: &'static str = "double";
}

#[cfg(not(feature = "fp_double_precision"))]
mod rtow_types {
    pub type Real = f32;
    pub const C_255_999: Real = 255.999f32;
    pub const C_256: Real = 256f32;
    pub const C_INFINITY: Real = std::f32::INFINITY;
    pub const C_PI: Real = std::f32::consts::PI;
    pub const C_ONE: Real = 1.0f32;
    pub const C_HALF_ONE: Real = 0.5f32;
    pub const C_TWO: Real = 2.0f32;
    pub const C_ZERO: Real = 0.0f32;
    pub const C_TWO_PI: Real = C_PI * C_TWO;
    pub const FP_MODEL: &'static str = "single";
}

use math::vec3::normalize;
use rand::Rng;
pub use rtow_types::*;
pub type Vec2 = math::vec2::TVec2<Real>;
pub type Vec3 = math::vec3::TVec3<Real>;
pub type Vec4 = math::vec4::TVec4<Real>;
pub type Ray = math::ray::TRay<Real>;
pub type Point = Vec3;
pub type Color = Vec3;
pub type Mat4 = math::mat4::Mat4<Real>;

pub fn degrees_to_radians(degrees: Real) -> Real {
    (degrees * C_PI) / 180 as Real
}

pub fn random_real() -> Real {
    rand::thread_rng().gen_range(0.0, 1.0) as Real
}

pub fn random_real_range(min: Real, max: Real) -> Real {
    use rand::prelude::*;
    thread_rng().gen_range(min, max)
}

pub fn clamp(x: Real, min: Real, max: Real) -> Real {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

pub fn rand_vec3() -> Vec3 {
    Vec3::new(random_real(), random_real(), random_real())
}

pub fn random_color() -> Color {
    Color::new(
        random_real_range(0 as Real, 1 as Real),
        random_real_range(0 as Real, 1 as Real),
        random_real_range(0 as Real, 1 as Real),
    )
}

pub fn random_color_in_range(min: Real, max: Real) -> Color {
    Color::new(
        random_real_range(min, max),
        random_real_range(min, max),
        random_real_range(min, max),
    )
}

pub fn rand_vec3_range(min: Real, max: Real) -> Vec3 {
    Vec3::new(
        random_real_range(min, max),
        random_real_range(min, max),
        random_real_range(min, max),
    )
}

pub fn random_in_unit_sphere() -> Vec3 {
    loop {
        let p = rand_vec3_range(-1 as Real, 1 as Real);
        if math::vec3::length_squared(p) >= 1 as Real {
            continue;
        }

        break p;
    }
}

pub fn random_int(min: i32, max: i32) -> i32 {
    rand::thread_rng().gen_range(min, max + 1)
}

pub fn random_unit_vector() -> Vec3 {
    normalize(random_in_unit_sphere())
}

pub fn random_in_hemisphere(normal: &Vec3) -> Vec3 {
    let in_unit_sphere = random_in_unit_sphere();
    use math::vec3::are_on_the_same_plane_side;

    if are_on_the_same_plane_side(in_unit_sphere, *normal) {
        in_unit_sphere
    } else {
        -in_unit_sphere
    }
}

pub fn random_in_unit_disk() -> Vec3 {
    loop {
        let v = Vec3::new(
            random_real_range(-1 as Real, 1 as Real),
            random_real_range(-1 as Real, 1 as Real),
            0 as Real,
        );

        if math::vec3::length_squared(v) >= 1 as Real {
            continue;
        }

        break v;
    }
}

pub fn random_cosine_direction() -> Vec3 {
    let r1 = random_real();
    let r2 = random_real();
    let z = (1 as Real - r2).sqrt();

    let phi = (2.0 * std::f64::consts::PI) as Real * r1;
    let x = phi.cos() * r2.sqrt();
    let y = phi.sin() * r2.sqrt();

    Vec3 { x, y, z }
}

pub fn random_to_sphere(radius: Real, distance_squared: Real) -> Vec3 {
    let r1 = random_real();
    let r2 = random_real();
    let z = 1 as Real + r2 * ((1 as Real - radius * radius / distance_squared).sqrt() - 1 as Real);

    let phi = (2.0 * std::f64::consts::PI) as Real * r1;
    let x = phi.cos() * (1 as Real - z * z).sqrt();
    let y = phi.sin() * (1 as Real - z * z).sqrt();

    Vec3 { x, y, z }
}

pub fn schlick(cosine: Real, refraction_index: Real) -> Real {
    let r0 = (1 as Real - refraction_index) / (1 as Real + refraction_index);
    let r0 = r0 * r0;
    r0 + (1 as Real - r0) * (1 as Real - cosine).powi(5)
}

pub fn ffmin(a: Real, b: Real) -> Real {
    if a < b {
        a
    } else {
        b
    }
}

pub fn ffmax(a: Real, b: Real) -> Real {
    if a > b {
        a
    } else {
        b
    }
}

/// Generates a random rotation matrix. Based on Graphics Gems 2, pg. 378
pub fn random_rotation_matrix() -> Mat4 {
    let [x1, x2, x3] = [random_real(), random_real(), random_real()];

    let z = x1;
    let phi = C_TWO_PI * x2;
    let r = (C_ONE - z * z).sqrt();
    let w = C_PI * x3;

    let (sin_phi, cos_phi) = phi.sin_cos();
    let (sin_w, cos_w) = w.sin_cos();

    let a = cos_w;
    let b = sin_w * cos_phi * r;
    let c = sin_w * sin_phi * r;
    let d = sin_w * z;

    Mat4 {
        a00: C_ONE - C_TWO * (c * c + d * d),
        a01: C_TWO * (b * c + a * d),
        a02: C_TWO * (b * d - a * c),
        a03: C_ZERO,

        //
        //
        a10: C_TWO * (b * c - a * d),
        a11: C_ONE - C_TWO * (b * b + d * d),
        a12: C_TWO * (c * d + a * b),
        a13: C_ZERO,

        //
        //
        a20: C_TWO * (b * d + a * c),
        a21: C_TWO * (c * d - a * b),
        a22: C_ONE - C_TWO * (b * b + c * c),
        a23: C_ZERO,

        //
        //
        a30: C_ZERO,
        a31: C_ZERO,
        a32: C_ZERO,
        a33: C_ONE,
    }
}
