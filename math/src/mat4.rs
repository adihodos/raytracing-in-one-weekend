use super::mat2x3::Mat2X3;
use super::vec4::TVec4;
use crate::vec3::TVec3;
use num::Float;
use num_traits::Num;

/// A 4x4 matrix, stored in row major ordering.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Mat4<T> {
    pub a00: T,
    pub a01: T,
    pub a02: T,
    pub a03: T,

    pub a10: T,
    pub a11: T,
    pub a12: T,
    pub a13: T,

    pub a20: T,
    pub a21: T,
    pub a22: T,
    pub a23: T,

    pub a30: T,
    pub a31: T,
    pub a32: T,
    pub a33: T,
}

impl<T> Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(&self.a00 as *const _, 16) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(&mut self.a00 as *mut _, 16) }
    }

    pub fn as_ptr(&self) -> *const T {
        &self.a00 as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        &mut self.a00 as *mut _
    }

    pub fn transpose(&self) -> Self {
        Self {
            a00: self.a00,
            a01: self.a10,
            a02: self.a20,
            a03: self.a30,

            a10: self.a01,
            a11: self.a11,
            a12: self.a21,
            a13: self.a31,

            a20: self.a02,
            a21: self.a12,
            a22: self.a22,
            a23: self.a32,

            a30: self.a03,
            a31: self.a13,
            a32: self.a23,
            a33: self.a33,
        }
    }

    pub fn translate(p: TVec3<T>) -> Self
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        Self {
            a00: T::one(),
            a01: T::zero(),
            a02: T::zero(),
            a03: p.x,

            a10: T::zero(),
            a11: T::one(),
            a12: T::zero(),
            a13: p.y,

            a20: T::zero(),
            a21: T::zero(),
            a22: T::one(),
            a23: p.z,

            a30: T::zero(),
            a31: T::zero(),
            a32: T::zero(),
            a33: T::one(),
        }
    }

    pub fn non_uniform_scale(s: TVec3<T>) -> Self {
        Self {
            a00: s.x,
            a01: T::zero(),
            a02: T::zero(),
            a03: T::zero(),

            a10: T::zero(),
            a11: s.y,
            a12: T::zero(),
            a13: T::zero(),

            a20: T::zero(),
            a21: T::zero(),
            a22: s.z,
            a23: T::zero(),

            a30: T::zero(),
            a31: T::zero(),
            a32: T::zero(),
            a33: T::one(),
        }
    }

    pub fn uniform_scale(s: T) -> Self {
        Self::non_uniform_scale((s, s, s).into())
    }

    pub fn column(&self, idx: usize) -> TVec4<T>
    where
        T: Num,
    {
        assert!(idx < 4);
        let s = self.as_slice();

        // a00 a01 a02 a03
        // a10 a11 a12 a13
        // a20 a21 a22 a23
        // a30 a31 a32 a33

        TVec4 {
            x: s[idx],
            y: s[idx + 4],
            z: s[idx + 8],
            w: s[idx + 12],
        }
    }
}

pub mod consts {
    use super::Mat4;
    use num_traits::Num;

    pub fn null<T>() -> Mat4<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        Mat4 {
            a00: T::zero(),
            a01: T::zero(),
            a02: T::zero(),
            a03: T::zero(),

            a10: T::zero(),
            a11: T::zero(),
            a12: T::zero(),
            a13: T::zero(),

            a20: T::zero(),
            a21: T::zero(),
            a22: T::zero(),
            a23: T::zero(),

            a30: T::zero(),
            a31: T::zero(),
            a32: T::zero(),
            a33: T::zero(),
        }
    }

    pub fn identity<T>() -> Mat4<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        Mat4 {
            a00: T::one(),
            a11: T::one(),
            a22: T::one(),
            a33: T::one(),
            ..null()
        }
    }
}

impl<T> std::ops::Deref for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> std::ops::DerefMut for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T> std::convert::AsRef<[T]> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> std::convert::AsMut<[T]> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> std::borrow::Borrow<[T]> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> std::borrow::BorrowMut<[T]> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> std::iter::FromIterator<T> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut m = std::mem::MaybeUninit::<Mat4<T>>::uninit();
        iter.into_iter().enumerate().for_each(|(idx, val)| unsafe {
            (m.as_mut_ptr() as *mut T).add(idx).write(val);
        });

        unsafe { m.assume_init() }
    }
}

impl<T> std::convert::From<[T; 16]> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn from(arr: [T; 16]) -> Self {
        unsafe {
            let mut m = std::mem::MaybeUninit::<Self>::uninit();
            std::ptr::copy_nonoverlapping(arr.as_ptr(), m.as_mut_ptr() as *mut _, 16);
            m.assume_init()
        }
    }
}

impl<T> std::convert::From<[[T; 4]; 4]> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn from(arr: [[T; 4]; 4]) -> Self {
        Self {
            //
            //
            a00: arr[0][0],
            a01: arr[0][1],
            a02: arr[0][2],
            a03: arr[0][3],

            //
            //
            a10: arr[1][0],
            a11: arr[1][1],
            a12: arr[1][2],
            a13: arr[1][3],

            //
            //
            a20: arr[2][0],
            a21: arr[2][1],
            a22: arr[2][2],
            a23: arr[2][3],

            //
            //
            a30: arr[3][0],
            a31: arr[3][1],
            a32: arr[3][2],
            a33: arr[3][3],
        }
    }
}

impl<T> std::convert::From<Mat2X3<T>> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn from(m: Mat2X3<T>) -> Self {
        Self {
            a00: m.a00,
            a01: m.a01,
            a02: T::zero(),
            a03: m.a02,

            a10: m.a10,
            a11: m.a11,
            a12: T::zero(),
            a13: m.a12,

            ..consts::identity()
        }
    }
}

impl<T> std::ops::Index<usize> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    type Output = TVec4<T>;

    fn index(&self, idx: usize) -> &Self::Output {
        debug_assert!(idx < 4);

        unsafe { &*(self.as_ptr().add(idx * 4) as *const TVec4<T>) }
    }
}

impl<T> std::ops::IndexMut<usize> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug,
{
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        debug_assert!(idx < 4);

        unsafe { &mut *(self.as_mut_ptr().add(idx * 4) as *mut TVec4<T>) }
    }
}

impl<T> std::ops::AddAssign for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.as_mut_slice()
            .iter_mut()
            .zip(rhs.as_slice().iter())
            .for_each(|(dst, src)| {
                *dst += *src;
            });
    }
}

impl<T> std::ops::Add for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        use std::iter::FromIterator;

        Self::from_iter(
            self.as_slice()
                .iter()
                .zip(rhs.as_slice().iter())
                .map(|(a, b)| *a + *b),
        )
    }
}

impl<T> std::ops::SubAssign for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.as_mut_slice()
            .iter_mut()
            .zip(rhs.as_slice().iter())
            .for_each(|(dst, src)| {
                *dst -= *src;
            });
    }
}

impl<T> std::ops::MulAssign<T> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::MulAssign,
{
    fn mul_assign(&mut self, scalar: T) {
        self.as_mut_slice()
            .iter_mut()
            .for_each(|dst| *dst *= scalar);
    }
}

impl<T> std::ops::Mul<T> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::Mul<Output = T>,
{
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        use std::iter::FromIterator;
        Self::from_iter(self.as_slice().iter().map(|e| *e * scalar))
    }
}

impl<T> std::ops::Mul<TVec4<T>> for Mat4<T>
where
    T: Num + Copy + Clone + std::ops::Mul<Output = T> + std::ops::Add<Output = T>,
{
    type Output = TVec4<T>;

    fn mul(self, rhs: TVec4<T>) -> Self::Output {
        TVec4 {
            x: self.a00 * rhs.x + self.a01 * rhs.y + self.a02 * rhs.z + self.a03 * rhs.w,
            y: self.a10 * rhs.x + self.a11 * rhs.y + self.a12 * rhs.z + self.a13 * rhs.w,
            z: self.a20 * rhs.x + self.a21 * rhs.y + self.a22 * rhs.z + self.a23 * rhs.w,
            w: rhs.w,
        }
    }
}

///  Macro to generate scalar with Mat4 multiplication
macro_rules! scalar_multiply_mat4 {
    ($stype:ty) => {
        impl std::ops::Mul<Mat4<$stype>> for $stype {
            type Output = Mat4<$stype>;

            fn mul(self, rhs: Mat4<$stype>) -> Self::Output {
                rhs * self
            }
        }
    };
}

scalar_multiply_mat4!(i8);
scalar_multiply_mat4!(u8);
scalar_multiply_mat4!(i16);
scalar_multiply_mat4!(u16);
scalar_multiply_mat4!(i32);
scalar_multiply_mat4!(u32);
scalar_multiply_mat4!(i64);
scalar_multiply_mat4!(u64);
scalar_multiply_mat4!(f32);
scalar_multiply_mat4!(f64);

impl<T> std::ops::DivAssign<T> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::DivAssign,
{
    fn div_assign(&mut self, scalar: T) {
        self.as_mut_slice()
            .iter_mut()
            .for_each(|dst| *dst /= scalar);
    }
}

impl<T> std::ops::Div<T> for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::Div<Output = T>,
{
    type Output = Self;
    fn div(self, scalar: T) -> Self::Output {
        use std::iter::FromIterator;
        Self::from_iter(self.as_slice().iter().map(|e| *e / scalar))
    }
}

impl<T> std::ops::Mul for Mat4<T>
where
    T: Num + Copy + Clone + std::fmt::Debug + std::ops::AddAssign + std::ops::Mul,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = consts::null();

        (0..4).for_each(|row| {
            (0..4).for_each(|col| {
                (0..4).for_each(|k| {
                    res[row][col] += self[row][k] * rhs[k][col];
                });
            });
        });

        res
    }
}

pub type Mat4F32 = Mat4<f32>;
pub type Mat4I32 = Mat4<i32>;

pub fn det<T: Float>(m: &Mat4<T>) -> T {
    //
    // This computation is based on Laplace's theorem
    // which states that the value of a determinant is equal to the product of
    // the minor determinants formed with the elements of p rows/columns and
    // their algebraic complements.
    let k1 = m.a00 * m.a11 - m.a01 * m.a10;
    let l1 = m.a22 * m.a33 - m.a23 * m.a32;

    let k2 = m.a00 * m.a12 - m.a02 * m.a10;
    let l2 = m.a21 * m.a33 - m.a12 * m.a31;

    let k3 = m.a00 * m.a13 - m.a03 * m.a10;
    let l3 = m.a21 * m.a32 - m.a22 * m.a31;

    let k4 = m.a01 * m.a12 - m.a02 * m.a11;
    let l4 = m.a20 * m.a33 - m.a32 * m.a30;

    let k5 = m.a01 * m.a13 - m.a03 * m.a11;
    let l5 = m.a20 * m.a32 - m.a22 * m.a30;

    let k6 = m.a02 * m.a13 - m.a03 * m.a12;
    let l6 = m.a20 * m.a31 - m.a21 * m.a30;

    k1 * l1 - k2 * l2 + k3 * l3 + k4 * l4 - k5 * l5 + k6 * l6
}

pub fn is_invertible<T: Float>(m: &Mat4<T>) -> bool {
    !det(m).is_zero()
}

pub fn adjoint<T: Float>(m: &Mat4<T>) -> Mat4<T> {
    let m1 = m.a22 * m.a33 - m.a23 * m.a32;
    let m2 = m.a21 * m.a33 - m.a23 * m.a31;
    let m3 = m.a21 * m.a32 - m.a22 * m.a31;
    let m4 = m.a02 * m.a13 - m.a03 * m.a12;
    let m5 = m.a01 * m.a13 - m.a03 * m.a11;
    let m6 = m.a01 * m.a12 - m.a02 * m.a11;
    let m7 = m.a20 * m.a33 - m.a23 * m.a30;
    let m8 = m.a20 * m.a32 - m.a22 * m.a30;
    let m9 = m.a12 * m.a33 - m.a13 * m.a32;
    let m10 = m.a10 * m.a33 - m.a13 * m.a30;
    let m11 = m.a10 * m.a32 - m.a12 * m.a30;
    let m12 = m.a00 * m.a13 - m.a03 * m.a10;
    let m13 = m.a00 * m.a12 - m.a02 * m.a10;
    let m14 = m.a20 * m.a31 - m.a21 * m.a30;
    let m15 = m.a00 * m.a11 - m.a01 * m.a10;
    let m16 = m.a20 * m.a31 - m.a21 * m.a30;

    Mat4 {
        a00: m.a11 * m1 - m.a12 * m2 + m.a13 * m3,
        a01: -m.a01 * m1 + m.a02 * m2 - m.a03 * m3,
        a02: m.a31 * m4 - m.a32 * m5 + m.a33 * m6,
        a03: -m.a21 * m4 + m.a22 * m5 - m.a23 * m6,

        a10: -m.a10 * m1 + m.a12 * m7 - m.a13 * m8,
        a11: m.a00 * m1 - m.a02 * m7 + m.a03 * m8,
        a12: -m.a00 * m9 + m.a02 * m10 - m.a03 * m11,
        a13: m.a20 * m4 - m.a22 * m12 + m.a23 * m13,

        a20: m.a10 * m2 - m.a11 * m7 + m.a13 * m16,
        a21: -m.a00 * m2 + m.a01 * m7 - m.a03 * m16,
        a22: m.a30 * m5 - m.a31 * m12 + m.a33 * m15,
        a23: -m.a20 * m5 + m.a21 * m12 - m.a23 * m15,

        a30: -m.a10 * m3 + m.a11 * m8 - m.a12 * m14,
        a31: m.a00 * m3 - m.a01 * m8 + m.a02 * m14,
        a32: -m.a30 * m6 + m.a31 * m13 - m.a32 * m15,
        a33: m.a20 * m6 - m.a21 * m13 + m.a22 * m15,
    }
}

pub fn invert<T: Float + std::fmt::Debug>(m: &Mat4<T>) -> Mat4<T> {
    let d = det(m);
    assert!(!d.is_zero(), "Matrix is not invertible");

    if d.is_zero() {
        *m
    } else {
        adjoint(m) * d.recip()
    }
}

#[cfg(test)]
mod tests {
    use super::super::vec4::*;
    use super::*;

    #[test]
    fn test_index_ops() {
        use std::convert::From;
        use std::iter::FromIterator;

        let m = Mat4::from_iter(0..16);

        assert_eq!(m[0], Vec4I32::new(0, 1, 2, 3));
        assert_eq!(m[1], Vec4I32::new(4, 5, 6, 7));
        assert_eq!(m[2], Vec4I32::new(8, 9, 10, 11));
        assert_eq!(m[3], Vec4I32::new(12, 13, 14, 15));

        let mut m = Mat4::from_iter(0..16);
        m[0].as_mut_slice().iter_mut().for_each(|x| *x *= 2);
        assert_eq!(m[0], Vec4I32::from([0, 2, 4, 6]));
    }

    #[test]
    fn test_multiplication() {
        use std::iter::FromIterator;
        let m0 = Mat4::from_iter(1..=16);
        let m1 = Mat4::from_iter(17..=17 + 15);

        let res = m0 * m1;
        assert_eq!(
            res,
            Mat4::from([
                250, 260, 270, 280, 618, 644, 670, 696, 986, 1028, 1070, 1112, 1354, 1412, 1470,
                1528
            ])
        );
    }

    #[test]
    fn test_transpose() {
        use std::iter::FromIterator;
        let m = Mat4::from_iter(0..16);
        let m1 = m.transpose();

        assert_eq!(
            m1,
            Mat4::from([0, 4, 8, 12, 1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15])
        );
    }
}
