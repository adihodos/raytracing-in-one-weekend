use super::vec3::TVec3;
use num_traits::{Float, Num};
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};

/// Four component vector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct TVec4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T> TVec4<T>
where
    T: Copy + Clone + std::fmt::Debug + Num,
{
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        TVec4 { x, y, z, w }
    }

    pub fn from_vec3(v: &TVec3<T>, w: T) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }

    pub fn same(t: T) -> Self {
        Self::new(t, t, t, t)
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const T, 4) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self as *mut Self as *mut T, 4) }
    }

    pub fn square_len(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn len(&self) -> T
    where
        T: Float,
    {
        self.square_len().sqrt()
    }
}

pub mod consts {
    use super::TVec4;
    use num::Num;

    pub fn null<T>() -> TVec4<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        TVec4::new(T::zero(), T::zero(), T::zero(), T::zero())
    }
}

impl<T> std::default::Default for TVec4<T>
where
    T: Copy + Clone + std::fmt::Debug + Num,
{
    fn default() -> Self {
        Self::new(T::zero(), T::zero(), T::zero(), T::zero())
    }
}

impl<T> Into<(T, T, T, T)> for TVec4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn into(self) -> (T, T, T, T) {
        (self.x, self.y, self.z, self.w)
    }
}

/// Subscripting operator - non mutable
impl<T> Index<usize> for TVec4<T>
where
    T: Copy + Clone + std::fmt::Debug + Num,
{
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.as_slice()[idx]
    }
}

/// Subscripting operator - mutable
impl<T> IndexMut<usize> for TVec4<T>
where
    T: Copy + Clone + std::fmt::Debug + Num,
{
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[idx]
    }
}

impl<T> std::iter::FromIterator<T> for TVec4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut v = consts::null();
        v.as_mut_slice()
            .iter_mut()
            .zip(iter)
            .for_each(|(dst, src)| *dst = src);
        v
    }
}

impl<T, U: AsRef<[T]>> std::convert::From<U> for TVec4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn from(s: U) -> Self {
        let mut v = consts::null();
        v.as_mut_slice()
            .iter_mut()
            .zip(s.as_ref().iter())
            .for_each(|(dst, src)| *dst = *src);
        v
    }
}

/// \brief  Negation operator.
impl<T> Neg for TVec4<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }
}

/// Self-assignment +
impl<T> AddAssign for TVec4<T>
where
    T: Copy + Clone + Num + AddAssign<T> + std::fmt::Debug,
{
    fn add_assign(&mut self, rhs: TVec4<T>) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

/// Binary +
impl<T> Add for TVec4<T>
where
    T: Copy + Clone + Num + AddAssign + std::fmt::Debug,
{
    type Output = TVec4<T>;

    fn add(self, rhs: TVec4<T>) -> Self::Output {
        let mut res = self;
        res += rhs;
        res
    }
}

/// Self-assign -
impl<T> SubAssign for TVec4<T>
where
    T: Copy + Clone + Num + SubAssign<T> + std::fmt::Debug,
{
    fn sub_assign(&mut self, rhs: TVec4<T>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

/// Binary -
impl<T> Sub for TVec4<T>
where
    T: Copy + Clone + Num + SubAssign<T> + std::fmt::Debug,
{
    type Output = TVec4<T>;

    fn sub(self, rhs: TVec4<T>) -> Self::Output {
        let mut res = self;
        res -= rhs;
        res
    }
}

/// Self-assign multiplication with scalar
impl<T> MulAssign<T> for TVec4<T>
where
    T: Copy + Clone + Num + MulAssign<T> + std::fmt::Debug,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

/// Self assign component-wise multiplication
impl<T> MulAssign for TVec4<T>
where
    T: Copy + Clone + Num + MulAssign<T> + std::fmt::Debug,
{
    fn mul_assign(&mut self, rhs: TVec4<T>) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self.w *= rhs.w;
    }
}

/// Binary multiplication with scalar
impl<T> Mul<T> for TVec4<T>
where
    T: Copy + Clone + Num + MulAssign<T> + std::fmt::Debug,
{
    type Output = TVec4<T>;

    fn mul(self, rhs: T) -> Self::Output {
        let mut res = self;
        res *= rhs;
        res
    }
}

/// Binary component-wise multiplication
impl<T> Mul<TVec4<T>> for TVec4<T>
where
    T: Copy + Clone + Num + MulAssign<T> + std::fmt::Debug,
{
    type Output = TVec4<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = self;
        res *= rhs;
        res
    }
}

gen_multiply_vector_to_scalar_ops!(TVec4<i8>, i8);
gen_multiply_vector_to_scalar_ops!(TVec4<u8>, u8);
gen_multiply_vector_to_scalar_ops!(TVec4<i16>, i16);
gen_multiply_vector_to_scalar_ops!(TVec4<u16>, u16);
gen_multiply_vector_to_scalar_ops!(TVec4<i32>, i32);
gen_multiply_vector_to_scalar_ops!(TVec4<u32>, u32);
gen_multiply_vector_to_scalar_ops!(TVec4<i64>, i64);
gen_multiply_vector_to_scalar_ops!(TVec4<u64>, u64);

pub type Vec4I8 = TVec4<i8>;
pub type Vec4U8 = TVec4<u8>;
pub type Vec4I16 = TVec4<i16>;
pub type Vec4U16 = TVec4<u16>;
pub type Vec4I32 = TVec4<i32>;
pub type Vec4U32 = TVec4<u32>;
pub type Vec4F32 = TVec4<f32>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::From;

    #[test]
    fn test_conversions() {
        let s = [0, 1, 2, 3];
        let v = Vec4I32::from(s);

        assert_eq!(v[0], s[0]);
        assert_eq!(v[1], s[1]);
        assert_eq!(v[2], s[2]);
        assert_eq!(v[3], s[3]);
    }
}
