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

    /// Set all components to the specified value
    pub fn broadcast(val: T) -> Self {
        TVec3 {
            x: val,
            y: val,
            z: val,
        }
    }

    /// Set x to specified value, y and z to default (0)
    pub fn with_x(x: T) -> Self {
        TVec3 {
            x,
            y: T::zero(),
            z: T::zero(),
        }
    }

    /// Set y to specified value, x and z to default (0)
    pub fn with_y(y: T) -> Self {
        TVec3 {
            x: T::zero(),
            y,
            z: T::zero(),
        }
    }

    /// Set z to specified value, x and y to default (0)
    pub fn with_z(z: T) -> Self {
        TVec3 {
            x: T::zero(),
            y: T::zero(),
            z,
        }
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

    // pub fn length_squared(&self) -> T {
    //     self.x * self.x + self.y * self.y + self.z * self.z
    // }

    // pub fn length(&self) -> T
    // where
    //     T: Float + Copy + Clone + std::fmt::Debug,
    // {
    //     self.length_squared().sqrt()
    // }

    // pub fn sqrt(&self) -> Self
    // where
    //     T: Float + Copy + Clone + std::fmt::Debug,
    // {
    //     Self {
    //         x: self.x.sqrt(),
    //         y: self.y.sqrt(),
    //         z: self.z.sqrt(),
    //     }
    // }
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

pub fn length_squared<T>(v: TVec3<T>) -> T
where
    T: Copy + Clone + Num + std::fmt::Debug + std::ops::Mul<Output = T> + std::ops::Add<Output = T>,
{
    v.x * v.x + v.y * v.y + v.z * v.z
}

pub fn length<T>(v: TVec3<T>) -> T
where
    T: Copy
        + Clone
        + Float
        + std::fmt::Debug
        + std::ops::Mul<Output = T>
        + std::ops::Add<Output = T>,
{
    length_squared(v).sqrt()
}

/// Component-wise squared root.
pub fn sqrt<T>(v: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Float + std::fmt::Debug,
{
    TVec3 {
        x: v.x.sqrt(),
        y: v.y.sqrt(),
        z: v.z.sqrt(),
    }
}

/// Component-wise boolean test.
pub fn test<T, F>(v: TVec3<T>, f: F) -> TVec3<bool>
where
    T: Copy + Clone + Num + std::fmt::Debug,
    F: Fn(T) -> bool,
{
    TVec3 {
        x: f(v.x),
        y: f(v.y),
        z: f(v.z),
    }
}

pub fn is_near_zero<T>(v: TVec3<T>) -> bool
where
    T: Copy + Clone + Float + std::fmt::Debug,
{
    length_squared(v).is_zero()
}

/// Make a unit length vector from the input vector.
pub fn normalize<T>(v: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Float + std::fmt::Debug,
{
    let lensq = length_squared(v);
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
    length_squared(v).is_one()
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

/// Reflect vector v along unit vector n.
pub fn reflect_unit_vector<T>(v: TVec3<T>, n: TVec3<T>) -> TVec3<T>
where
    T: Copy
        + Clone
        + Num
        + std::ops::Mul
        + std::ops::Add
        + std::ops::Sub
        + std::fmt::Debug
        + std::ops::Mul<TVec3<T>, Output = TVec3<T>>,
{
    let two = T::one() + T::one();
    v - two * dot(v, n) * n
}

/// Refract vector uv along normal n, given the ratio of refraction indices
pub fn refract<T>(uv: TVec3<T>, n: TVec3<T>, etai_over_etat: T) -> TVec3<T>
where
    T: Copy
        + Clone
        + Float
        + std::ops::Mul
        + std::ops::Add
        + std::ops::Sub
        + std::fmt::Debug
        + std::ops::Mul<TVec3<T>, Output = TVec3<T>>,
{
    let cos_theta = dot(-uv, n);
    let r_out_parallel = etai_over_etat * (uv + cos_theta * n);
    let r_out_perp = -(T::one() - length_squared(r_out_parallel)).sqrt() * n;
    r_out_parallel + r_out_perp
}

/// Test if two vectors are parallel using the cross product.
pub fn are_parallel<T>(a: TVec3<T>, b: TVec3<T>) -> bool
where
    T: Copy + Clone + Num + std::ops::Mul + std::ops::Add + std::ops::Sub + std::fmt::Debug,
{
    length_squared(cross(a, b)).is_zero()
}

pub fn angle_between<T>(a: TVec3<T>, b: TVec3<T>) -> T
where
    T: Copy + Clone + Float + std::fmt::Debug,
{
    (dot(a, b) / (length(a) * length(b))).acos()
}

/// Test if two vectors are on the same side of a half-space defined by a plane.
pub fn are_on_the_same_plane_side<T>(a: TVec3<T>, b: TVec3<T>) -> bool
where
    T: Copy
        + Clone
        + Num
        + std::ops::Mul
        + std::ops::Add
        + std::ops::Sub
        + std::fmt::Debug
        + std::cmp::PartialOrd,
{
    dot(a, b) > T::zero()
}

fn transform<T, F>(vec: TVec3<T>, f: F) -> TVec3<T>
where
    T: Copy + Clone,
    F: Fn(T) -> T,
{
    TVec3 {
        x: f(vec.x),
        y: f(vec.y),
        z: f(vec.z),
    }
}

pub fn sin<T>(angle: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Float + std::fmt::Debug,
{
    transform(angle, |x| x.sin())
}

pub fn cos<T>(angle: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Float + std::fmt::Debug,
{
    transform(angle, |x| x.cos())
}

pub fn tan<T>(angle: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Float + std::fmt::Debug,
{
    transform(angle, |x| x.tan())
}

pub fn abs<T>(val: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + num::Signed,
{
    transform(val, |x| x.abs())
}

pub fn sign<T>(val: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + num::Signed + std::cmp::PartialOrd,
{
    transform(val, |x| if x < T::zero() { -T::one() } else { T::one() })
}

pub fn min_sv<T>(a: TVec3<T>, b: T) -> TVec3<T>
where
    T: Copy + Clone + Num + crate::minmax::MinMax<Output = T>,
{
    TVec3 {
        x: T::min(a.x, b),
        y: T::min(a.y, b),
        z: T::min(a.z, b),
    }
}

pub fn min<T>(a: TVec3<T>, b: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Num + crate::minmax::MinMax<Output = T>,
{
    TVec3 {
        x: T::min(a.x, b.x),
        y: T::min(a.y, b.y),
        z: T::min(a.z, b.z),
    }
}

pub fn max_sv<T>(a: TVec3<T>, b: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Num + crate::minmax::MinMax<Output = T>,
{
    TVec3 {
        x: T::max(a.x, b.x),
        y: T::max(a.y, b.y),
        z: T::max(a.z, b.z),
    }
}

pub fn max<T>(a: TVec3<T>, b: T) -> TVec3<T>
where
    T: Copy + Clone + Num + crate::minmax::MinMax<Output = T>,
{
    TVec3 {
        x: T::max(a.x, b),
        y: T::max(a.y, b),
        z: T::max(a.z, b),
    }
}

pub fn clamp<T>(a: TVec3<T>, minval: TVec3<T>, maxval: TVec3<T>) -> TVec3<T>
where
    T: Copy + Clone + Num + crate::minmax::MinMax<Output = T>,
{
    TVec3 {
        x: T::min(T::max(a.x, minval.x), maxval.x),
        y: T::min(T::max(a.y, minval.y), maxval.y),
        z: T::min(T::max(a.z, minval.z), maxval.z),
    }
}

pub fn clamp_sv<T>(a: TVec3<T>, minval: T, maxval: T) -> TVec3<T>
where
    T: Copy + Clone + Num + crate::minmax::MinMax<Output = T>,
{
    TVec3 {
        x: T::min(T::max(a.x, minval), maxval),
        y: T::min(T::max(a.y, minval), maxval),
        z: T::min(T::max(a.z, minval), maxval),
    }
}

/// Returns the linear blend of x and y, i.e., x · (1 - a) + y · a
pub fn mix<T>(x: TVec3<T>, y: TVec3<T>, a: TVec3<T>) -> TVec3<T>
where
    T: Copy
        + Clone
        + Num
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>,
{
    TVec3 {
        x: (T::one() - a.x) * x.x + a.x * y.x,
        y: (T::one() - a.y) * x.y + a.y * y.y,
        z: (T::one() - a.z) * x.z + a.z * y.z,
    }
}

/// Returns the linear blend of x and y, i.e., x · (1 - a) + y · a
pub fn mix_sv<T>(x: TVec3<T>, y: TVec3<T>, a: T) -> TVec3<T>
where
    T: Copy
        + Clone
        + Num
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>,
{
    TVec3 {
        x: (T::one() - a) * x.x + a * y.x,
        y: (T::one() - a) * x.y + a * y.y,
        z: (T::one() - a) * x.z + a * y.z,
    }
}

/// Selects which vector each returned component
/// comes from. For a component of a that is false,
/// the corresponding component of x is returned.
/// For a component of a that is true, the
/// corresponding component of y is returned.
pub fn bmix<T>(x: TVec3<T>, y: TVec3<T>, a: TVec3<bool>) -> TVec3<T>
where
    T: Copy
        + Clone
        + Num
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>,
{
    TVec3 {
        x: if a.x { x.x } else { y.x },
        y: if a.y { x.y } else { y.y },
        z: if a.z { x.z } else { y.z },
    }
}
