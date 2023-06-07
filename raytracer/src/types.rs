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
    pub const FP_MODEL: &'static str = "single";
}

use math::vec3::normalize;
use rand::Rng;
pub use rtow_types::*;
pub type Vec3 = math::vec3::TVec3<Real>;
pub type Ray = math::ray::TRay<Real>;
pub type Point = Vec3;
pub type Color = Vec3;

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

pub fn schlick(cosine: Real, refraction_index: Real) -> Real {
    let r0 = (1 as Real - refraction_index) / (1 as Real + refraction_index);
    let r0 = r0 * r0;
    r0 + (1 as Real - r0) * (1 as Real - cosine).powi(5)
}
