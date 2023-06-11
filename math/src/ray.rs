use crate::mat4::Mat4;
use crate::vec3::TVec3;
use crate::vec4::TVec4;
use num::Float;
use num_traits::Num;

/// Ray in R3.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct TRay<T> {
    pub origin: TVec3<T>,
    pub direction: TVec3<T>,
    pub time: T,
}

impl<T> TRay<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    pub fn new(origin: TVec3<T>, direction: TVec3<T>, time: T) -> TRay<T> {
        TRay {
            origin,
            direction,
            time,
        }
    }

    pub fn at(&self, t: T) -> TVec3<T>
    where
        T: Copy
            + Clone
            + Num
            + std::fmt::Debug
            + std::ops::Add<Output = T>
            + std::ops::Mul<Output = T>
            + std::ops::Mul<TVec3<T>, Output = TVec3<T>>,
    {
        self.origin + t * self.direction
    }
}

pub fn transform<T>(mat: &Mat4<T>, ray: &TRay<T>) -> TRay<T>
where
    T: Float + std::fmt::Debug,
{
    let origin = (*mat * TVec4::from_vec3(&ray.origin, T::one())).xyz();
    let direction = (*mat * TVec4::from_vec3(&ray.direction, T::zero())).xyz();

    TRay {
        origin,
        direction,
        ..*ray
    }
}
