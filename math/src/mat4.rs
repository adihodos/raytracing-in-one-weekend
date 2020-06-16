use super::mat2x3::Mat2X3;
use super::vec4::TVec4;
use num_traits::Num;

/// A 4x4 matrix, stored in row major ordering.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Mat4<T> {
    a00: T,
    a01: T,
    a02: T,
    a03: T,

    a10: T,
    a11: T,
    a12: T,
    a13: T,

    a20: T,
    a21: T,
    a22: T,
    a23: T,

    a30: T,
    a31: T,
    a32: T,
    a33: T,
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
