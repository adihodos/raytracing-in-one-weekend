#[cfg(feature = "fp_double_precision")]
mod rtow_types {
    pub type Real = f64;
    pub const C_255_999: Real = 255.999f64;
    pub const C_256: Real = 256f64;
    pub const C_INFINITY: Real = std::f64::INFINITY;
    pub const C_PI: Real = std::f64::consts::PI;
}

#[cfg(not(feature = "fp_double_precision"))]
mod rtow_types {
    pub type Real = f32;
    pub const C_255_999: Real = 255.999f32;
    pub const C_256: Real = 256f32;
    pub const C_INFINITY: Real = std::f32::INFINITY;
    pub const C_PI: Real = std::f32::consts::PI;
}

pub use rtow_types::*;
pub type Vec3 = math::vec3::TVec3<Real>;
pub type Color = Vec3;
pub type Point = Vec3;
pub type Ray = math::ray::TRay<Real>;

pub fn degrees_to_radians(degrees: Real) -> Real {
    degrees * C_PI / 180f32
}

pub fn random_real() -> Real {
    use rand::prelude::*;
    thread_rng().gen()
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
