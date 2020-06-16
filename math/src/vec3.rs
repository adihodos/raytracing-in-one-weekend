use num_traits::{Float, Num};

/// Vector/point in R3.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct TVec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    pub fn new(x: T, y: T, z: T) -> Self {
        TVec3 { x, y, z }
    }

    pub fn same(val: T) -> Self {
        Self::new(val, val, val)
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(&self.x as *const _, 3) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(&mut self.x as *mut _, 3) }
    }

    pub fn as_ptr(&self) -> *const T {
        &self.x as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        &mut self.x as *mut _
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> T
    where
        T: Float + Copy + Clone + std::fmt::Debug,
    {
        self.length_squared().sqrt()
    }
}

pub mod consts {
    use super::TVec3;
    use num_traits::Num;

    pub fn null<T>() -> TVec3<T>
    where
        T: Copy + Clone + Num + std::fmt::Debug,
    {
        TVec3 {
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }

    pub fn unit_x<T>() -> TVec3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        TVec3 {
            x: T::one(),
            y: T::zero(),
            z: T::zero(),
        }
    }

    pub fn unit_y<T>() -> TVec3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        TVec3 {
            x: T::zero(),
            y: T::one(),
            z: T::zero(),
        }
    }

    pub fn unit_z<T>() -> TVec3<T>
    where
        T: Num + Copy + Clone + std::fmt::Debug,
    {
        TVec3 {
            x: T::zero(),
            y: T::zero(),
            z: T::one(),
        }
    }
}

impl<T> std::ops::Deref for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> std::ops::DerefMut for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T> std::convert::AsRef<[T]> for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> std::convert::AsMut<[T]> for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> std::borrow::Borrow<[T]> for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> std::borrow::BorrowMut<[T]> for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> std::convert::From<[T; 3]> for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    fn from(arr: [T; 3]) -> Self {
        TVec3 {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
}

impl<T> std::convert::From<(T, T, T)> for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    fn from(tpl: (T, T, T)) -> Self {
        TVec3 {
            x: tpl.0,
            y: tpl.1,
            z: tpl.2,
        }
    }
}

impl<T> std::convert::From<TVec3<T>> for (T, T, T)
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    fn from(v: TVec3<T>) -> Self {
        (v.x, v.y, v.z)
    }
}

impl<T> std::ops::Index<usize> for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        debug_assert!(idx < 3);
        &self.as_slice()[idx]
    }
}

impl<T> std::ops::IndexMut<usize> for TVec3<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        debug_assert!(idx < 3);
        &mut self.as_mut_slice()[idx]
    }
}

impl<T> std::ops::Neg for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::Neg<Output = T> + std::fmt::Debug,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl<T> std::ops::AddAssign for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::AddAssign + std::fmt::Debug,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<T> std::ops::Add for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::Add<Output = T> + std::fmt::Debug,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> std::ops::SubAssign for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::SubAssign + std::fmt::Debug,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<T> std::ops::Sub for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::Sub<Output = T> + std::fmt::Debug,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> std::ops::MulAssign<T> for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::MulAssign + std::fmt::Debug,
{
    fn mul_assign(&mut self, k: T) {
        self.x *= k;
        self.y *= k;
        self.z *= k;
    }
}

/// Component-wise self assign multiplication with another TVec3.
impl<T> std::ops::MulAssign for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::MulAssign + std::fmt::Debug,
{
    fn mul_assign(&mut self, v: TVec3<T>) {
        self.x *= v.x;
        self.y *= v.y;
        self.z *= v.z;
    }
}

impl<T> std::ops::Mul<T> for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::Mul<Output = T> + std::fmt::Debug,
{
    type Output = Self;

    fn mul(self, k: T) -> Self::Output {
        Self {
            x: self.x * k,
            y: self.y * k,
            z: self.z * k,
        }
    }
}

///  Macro to generate scalar with TVec3 multiplication
macro_rules! scalar_multiply_tvec3 {
    ($stype:ty) => {
        impl std::ops::Mul<TVec3<$stype>> for $stype {
            type Output = TVec3<$stype>;

            fn mul(self, rhs: TVec3<$stype>) -> Self::Output {
                rhs * self
            }
        }
    };
}

scalar_multiply_tvec3!(i8);
scalar_multiply_tvec3!(u8);
scalar_multiply_tvec3!(i16);
scalar_multiply_tvec3!(u16);
scalar_multiply_tvec3!(i32);
scalar_multiply_tvec3!(u32);
scalar_multiply_tvec3!(i64);
scalar_multiply_tvec3!(u64);
scalar_multiply_tvec3!(f32);
scalar_multiply_tvec3!(f64);

/// Component-wise multiplication with another TVec3.
impl<T> std::ops::Mul for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::Mul<Output = T> + std::fmt::Debug,
{
    type Output = Self;
    fn mul(self, v: TVec3<T>) -> Self::Output {
        Self {
            x: self.x * v.x,
            y: self.y * v.y,
            z: self.z * v.z,
        }
    }
}

impl<T> std::ops::DivAssign<T> for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::DivAssign + std::fmt::Debug,
{
    fn div_assign(&mut self, k: T) {
        self.x /= k;
        self.y /= k;
        self.z /= k;
    }
}

/// Component-wise self assign division with another TVec3
impl<T> std::ops::DivAssign for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::DivAssign + std::fmt::Debug,
{
    fn div_assign(&mut self, rhs: TVec3<T>) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z
    }
}

impl<T> std::ops::Div<T> for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::Div<Output = T> + std::fmt::Debug,
{
    type Output = Self;
    fn div(self, k: T) -> Self::Output {
        Self {
            x: self.x / k,
            y: self.y / k,
            z: self.z / k,
        }
    }
}

/// Component-wise division with another TVec3
impl<T> std::ops::Div for TVec3<T>
where
    T: Copy + Clone + Num + std::ops::Div<Output = T> + std::fmt::Debug,
{
    type Output = Self;
    fn div(self, rhs: TVec3<T>) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

/// Make a unit length vector from the input vector.
pub fn normalize<T>(v: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Float + std::fmt::Debug,
{
    let lensq = v.length_squared();
    if lensq.is_zero() {
        consts::null()
    } else {
        v * lensq.sqrt().recip()
    }
}

/// Test if the input vector is unit length.
pub fn is_unit_length<T>(v: TVec3<T>) -> bool
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
    v.length_squared().is_one()
}

/// Dot product of two vectors.
pub fn dot<T>(a: TVec3<T>, b: TVec3<T>) -> T
where
    T: Copy + Clone + Num + std::ops::Mul + std::ops::Add + std::fmt::Debug,
{
    a.x * b.x + a.y * b.y + a.z * b.z
}

/// Test if two vectors are perpendicular to each other.
pub fn are_orthogonal<T>(a: TVec3<T>, b: TVec3<T>) -> bool
where
    T: Copy + Clone + Num + std::ops::Mul + std::ops::Add + std::fmt::Debug,
{
    dot(a, b).is_zero()
}

/// Cross product between two vectors. This will return a vector that is
/// orthogonal to both input vectors.
pub fn cross<T>(a: TVec3<T>, b: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Num + std::ops::Mul + std::ops::Add + std::ops::Sub + std::fmt::Debug,
{
    TVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

/// Test if two vectors are parallel using the cross product.
pub fn are_parallel<T>(a: TVec3<T>, b: TVec3<T>) -> bool
where
    T: Copy + Clone + Num + std::ops::Mul + std::ops::Add + std::ops::Sub + std::fmt::Debug,
{
    cross(a, b).length_squared().is_zero()
}

pub fn angle_between<T>(a: TVec3<T>, b: TVec3<T>) -> T
where
    T: Copy + Clone + Float + std::fmt::Debug,
{
    (dot(a, b) / (a.length() * b.length())).acos()
}
