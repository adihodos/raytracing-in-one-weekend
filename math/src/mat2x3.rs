use super::vec2::TVec2;
use super::vec3::TVec3;
use num_traits::{Float, Num};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    pub a00: T,
    pub a01: T,
    pub a02: T,
    pub a10: T,
    pub a11: T,
    pub a12: T,
}

impl<T> Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    pub fn new(a00: T, a01: T, a02: T, a10: T, a11: T, a12: T) -> Self {
        Mat2X3 {
            a00,
            a01,
            a02,
            a10,
            a11,
            a12,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(&self.a00 as *const _, 6) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(&mut self.a00 as *mut _, 6) }
    }

    pub fn as_ptr(&self) -> *const T {
        &self.a00 as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        &mut self.a00 as *mut _
    }
}

pub mod consts {
    use num_traits::Num;

    pub fn null<T>() -> super::Mat2X3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        super::Mat2X3 {
            a00: T::zero(),
            a01: T::zero(),
            a02: T::zero(),
            a10: T::zero(),
            a11: T::zero(),
            a12: T::zero(),
        }
    }

    pub fn identity<T>() -> super::Mat2X3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        super::Mat2X3 {
            a00: T::one(),
            a01: T::zero(),
            a02: T::zero(),
            a10: T::zero(),
            a11: T::one(),
            a12: T::zero(),
        }
    }
}

impl<T> std::iter::FromIterator<T> for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut m = consts::null();
        m.as_mut_slice()
            .iter_mut()
            .zip(iter)
            .for_each(|(dst, src)| *dst = src);

        m
    }
}

impl<T, U: AsRef<[T]>> std::convert::From<U> for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn from(s: U) -> Self {
        let mut m = consts::null();
        m.as_mut_slice()
            .iter_mut()
            .zip(s.as_ref().iter())
            .for_each(|(dst, src)| {
                *dst = *src;
            });
        m
    }
}

impl<T> std::ops::Index<usize> for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    type Output = TVec3<T>;

    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => unsafe { &*(&self.a00 as *const _ as *const TVec3<T>) },
            1 => unsafe { &*(&self.a10 as *const _ as *const TVec3<T>) },
            _ => panic!("Row index out of bounds!"),
        }
    }
}

impl<T> std::ops::IndexMut<usize> for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        debug_assert!(idx < 2, "Row out of bounds!");
        match idx {
            0 => unsafe { &mut *(&mut self.a00 as *mut _ as *mut TVec3<T>) },
            1 => unsafe { &mut *(&mut self.a10 as *mut _ as *mut TVec3<T>) },
            _ => panic!("Row index out of bounds!"),
        }
    }
}

impl<T> std::ops::AddAssign for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.as_mut_slice()
            .iter_mut()
            .zip(rhs.as_slice().iter())
            .for_each(|(dst, src)| *dst += *src);
    }
}

impl<T> std::ops::Add for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::AddAssign,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut m = self;
        m += rhs;
        m
    }
}

impl<T> std::ops::SubAssign for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.as_mut_slice()
            .iter_mut()
            .zip(rhs.as_slice().iter())
            .for_each(|(dst, src)| *dst -= *src);
    }
}

impl<T> std::ops::Sub for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::SubAssign,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut m = self;
        m -= rhs;
        m
    }
}

impl<T> std::ops::MulAssign<T> for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::MulAssign,
{
    fn mul_assign(&mut self, rhs: T) {
        self.as_mut_slice().iter_mut().for_each(|e| *e *= rhs);
    }
}

impl<T> std::ops::Mul for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            a00: self.a00 * rhs.a00 + self.a01 * rhs.a10,
            a01: self.a00 * rhs.a01 + self.a01 * rhs.a11,
            a02: self.a00 * rhs.a02 + self.a01 * rhs.a12 + self.a02,
            a10: self.a10 * rhs.a00 + self.a11 * rhs.a10,
            a11: self.a10 * rhs.a01 + self.a11 * rhs.a11,
            a12: self.a10 * rhs.a02 + self.a11 * rhs.a12 + self.a12,
        }
    }
}

impl<T> std::ops::Mul<T> for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        use std::iter::FromIterator;
        Self::from_iter(self.as_slice().iter().map(|e| *e * scalar))
    }
}

impl<T> std::ops::Div<T> for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::DivAssign,
{
    type Output = Self;
    fn div(self, scalar: T) -> Self::Output {
        let mut m = self;
        m /= scalar;
        m
    }
}

impl<T> std::ops::DivAssign<T> for Mat2X3<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::DivAssign,
{
    fn div_assign(&mut self, scalar: T) {
        self.as_mut_slice().iter_mut().for_each(|e| *e /= scalar);
    }
}

/// Apply affine transformation to a point.
pub fn mul_point<T>(m: &Mat2X3<T>, p: TVec2<T>) -> TVec2<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    TVec2 {
        x: m.a00 * p.x + m.a01 * p.y + m.a02,
        y: m.a10 * p.x + m.a11 * p.y + m.a12,
    }
}

/// Apply transformation to a vector.
pub fn mul_vec<T>(m: &Mat2X3<T>, p: TVec2<T>) -> TVec2<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    TVec2 {
        x: m.a00 * p.x + m.a01 * p.y,
        y: m.a10 * p.x + m.a11 * p.y,
    }
}

pub type Mat2X3I32 = Mat2X3<i32>;
pub type Mat2X3U32 = Mat2X3<u32>;
pub type Mat2X3F32 = Mat2X3<f32>;
pub type Mat2X3F64 = Mat2X3<f64>;

pub mod transforms {
    use super::*;

    pub fn translate<T>(x: T, y: T) -> Mat2X3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        Mat2X3::new(T::one(), T::zero(), x, T::zero(), T::one(), y)
    }

    pub fn uniform_scale<T>(s: T) -> Mat2X3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        Mat2X3::new(s, T::zero(), T::zero(), T::zero(), s, T::zero())
    }

    pub fn scale<T>(sx: T, sy: T) -> Mat2X3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        Mat2X3::new(sx, T::zero(), T::zero(), T::zero(), sy, T::zero())
    }

    pub fn rotate<T>(angle: T) -> Mat2X3<T>
    where
        T: Float + Copy + Clone + std::fmt::Debug,
    {
        let ct = angle.cos();
        let st = angle.sin();

        Mat2X3::new(ct, -st, T::zero(), st, ct, T::zero())
    }

    pub fn rotate_off_center<T>(angle: T, cx: T, cy: T) -> Mat2X3<T>
    where
        T: Float + Copy + Clone + std::fmt::Debug,
    {
        let ct = angle.cos();
        let st = angle.cos();

        Mat2X3::new(
            ct,
            -st,
            cx * (T::one() - ct) + st * cy,
            st,
            ct,
            cy * (T::one() - ct) - st * cx,
        )
    }

    pub fn shear_x<T>(s: T) -> Mat2X3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        Mat2X3::new(T::one(), s, T::zero(), T::zero(), T::one(), T::zero())
    }

    pub fn shear_y<T>(s: T) -> Mat2X3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        Mat2X3::new(T::one(), T::zero(), T::zero(), s, T::one(), T::zero())
    }

    pub fn shear_xy<T>(sx: T, sy: T) -> Mat2X3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        Mat2X3::new(T::one(), sx, T::zero(), sy, T::one(), T::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() <= std::f32::EPSILON
    }

    #[test]
    fn test_slice_fns() {
        let m = Mat2X3::new(1f32, 2f32, 3f32, 4f32, 5f32, 6f32);
        let sm = m.as_slice();

        assert!(approx_eq(sm[0], m[0][0]));
        assert!(approx_eq(sm[1], m[0][1]));
        assert!(approx_eq(sm[2], m[0][2]));

        assert!(approx_eq(sm[3], m[1][0]));
        assert!(approx_eq(sm[4], m[1][1]));
        assert!(approx_eq(sm[5], m[1][2]));

        let mut m = m;
        m[0][0] = 10f32;
        m[1][1] = 20f32;

        assert_eq!(m.a00, 10f32);
        assert_eq!(m.a11, 20f32);
    }

    #[test]
    fn test_ctors() {
        use std::convert::From;
        use std::iter::FromIterator;

        let m = Mat2X3::<i32>::from_iter(0..=5);

        assert_eq!(m.as_slice(), &[0, 1, 2, 3, 4, 5]);

        let m = Mat2X3I32::from([0, 1, 2, 3, 4, 5]);
        assert_eq!(m.as_slice(), &[0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_mul_scalar() {
        use std::iter::FromIterator;
        let m = Mat2X3::<i32>::from_iter(0..=5);
        let m1 = m * 2;

        assert_eq!(m1.as_slice(), &[0, 2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_div_scalar() {
        use std::iter::FromIterator;
        let m = Mat2X3::from_iter((0..=5).map(|i| i * 2));
        let m1 = m / 2;

        assert_eq!(m1.as_slice(), &[0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_multiplication() {
        let m0 = Mat2X3::from([1, 2, 3, 4, 5, 6]);
        let m1 = Mat2X3::from([7, 8, 9, 10, 11, 12]);
        let res = m0 * m1;

        assert_eq!(res, Mat2X3::from([27, 30, 36, 78, 87, 102]));
    }
}
