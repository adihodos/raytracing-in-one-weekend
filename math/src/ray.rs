use crate::vec3::TVec3;
use num_traits::Num;

/// Ray in R3.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct TRay<T> {
    pub origin: TVec3<T>,
    pub direction: TVec3<T>,
}

impl<T> TRay<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    pub fn new(origin: TVec3<T>, direction: TVec3<T>) -> TRay<T> {
        TRay { origin, direction }
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
