use num_traits::{Float, Num};

use crate::mat4::Mat4;
use crate::vec3;
use crate::vec3::TVec3;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Quat<T> {
    pub w: T,
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Quat<T>
where
    T: Copy + Clone + Num + std::fmt::Debug,
{
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

    /// Constructs a Quat that represents the rotation of vector V1 into vector V2.
    pub fn vector_rotation(v1: TVec3<T>, v2: TVec3<T>) -> Self
    where
        T: Float + Copy + Clone + std::ops::Add<Output = T> + std::ops::Mul<Output = T>,
    {
        //
        // What's the explanation behind this method ?? I can't remember where
        // I've seen it first.

        use vec3::*;
        let bisector = normalize(v1 + v2);

        let w = dot(v1, bisector);

        let (x, y, z) = if !w.is_zero() {
            let axis = cross(v1, bisector);
            (axis.x, axis.y, axis.z)
        } else {
            if v1.x.abs() > v1.y.abs() {
                let k = (v1.x * v1.x + v1.z * v1.z).sqrt().recip();
                (-v1.z * k, T::zero(), v1.x * k)
            } else {
                let k = (v1.y * v1.y + v1.z * v1.z).sqrt().recip();
                (T::zero(), v1.z * k, -v1.y * k)
            }
        };

        Self { w, x, y, z }
    }

    pub fn axis_angle(angle: T, axis: TVec3<T>) -> Self
    where
        T: Float + Copy + Clone + std::ops::Add<Output = T> + std::ops::Mul<Output = T>,
    {
        use vec3::*;

        let lensquared = length_squared(axis);

        if lensquared.is_zero() {
            self::consts::identity()
        } else {
            let angle = angle.to_radians();
            let scale_factor = (angle / (T::one() + T::one())).sin() / lensquared.sqrt();
            Self {
                w: (angle / (T::one() + T::one())).cos(),
                x: axis.x * scale_factor,
                y: axis.y * scale_factor,
                z: axis.z * scale_factor,
            }
        }
    }
}

pub mod consts {
    use super::Quat;
    use num_traits::Num;

    pub fn null<T>() -> Quat<T>
    where
        T: Copy + Clone + Num + std::fmt::Debug,
    {
        Quat {
            w: T::zero(),
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }

    pub fn identity<T>() -> Quat<T>
    where
        T: Copy + Clone + Num + std::fmt::Debug,
    {
        Quat {
            w: T::one(),
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }
}

impl<T> std::convert::From<[T; 4]> for Quat<T>
where
    T: Float,
{
    fn from(arr: [T; 4]) -> Self {
        Self {
            w: arr[0],
            x: arr[1],
            y: arr[2],
            z: arr[3],
        }
    }
}

impl<T> std::convert::From<(T, T, T, T)> for Quat<T>
where
    T: Float,
{
    fn from(a: (T, T, T, T)) -> Self {
        Self {
            w: a.0,
            x: a.1,
            y: a.2,
            z: a.3,
        }
    }
}

impl<T> std::ops::Add<Quat<T>> for Quat<T>
where
    T: Copy + Clone + Num + std::fmt::Debug + std::ops::Add<Output = T>,
{
    type Output = Quat<T>;

    fn add(self, rhs: Quat<T>) -> Self::Output {
        Quat {
            w: self.w + rhs.w,
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> std::ops::AddAssign<Quat<T>> for Quat<T>
where
    T: Copy + Clone + Num + std::fmt::Debug + std::ops::AddAssign,
{
    fn add_assign(&mut self, rhs: Quat<T>) {
        self.w += rhs.w;
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.w;
    }
}

impl<T> std::ops::Sub<Quat<T>> for Quat<T>
where
    T: Copy + Clone + Num + std::fmt::Debug + std::ops::Sub<Output = T>,
{
    type Output = Quat<T>;

    fn sub(self, rhs: Quat<T>) -> Self::Output {
        Quat {
            w: self.w - rhs.w,
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> std::ops::SubAssign<Quat<T>> for Quat<T>
where
    T: Float + std::ops::SubAssign,
{
    fn sub_assign(&mut self, rhs: Quat<T>) {
        self.w -= rhs.w;
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.w;
    }
}

impl<T> std::ops::Neg for Quat<T>
where
    T: Copy + Clone + Num + std::ops::Neg<Output = T>,
{
    type Output = Quat<T>;

    fn neg(self) -> Self::Output {
        Quat {
            w: -self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T> std::ops::Mul<Quat<T>> for Quat<T>
where
    T: Float,
{
    type Output = Quat<T>;

    fn mul(self, rhs: Quat<T>) -> Self::Output {
        Quat {
            w: self.w * rhs.w - (self.x * rhs.x + self.y * rhs.y + self.z * rhs.z),
            x: self.w * rhs.x + rhs.w * self.x + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y + rhs.w * self.y + self.z * rhs.x - self.x * rhs.z,
            z: self.w * rhs.z + rhs.w * self.z + self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl<T> std::ops::Mul<TVec3<T>> for Quat<T>
where
    T: Float,
{
    type Output = TVec3<T>;

    fn mul(self, v: TVec3<T>) -> Self::Output {
        assert!(
            is_unit_length(self),
            "Rotating a vector needs a unit length Quat!"
        );

        let two: T = T::one() + T::one();

        let dotp = two * (self.x * v.x + self.y * v.y + self.z * v.z);

        let cross_mul = two * self.w;
        let vmul = cross_mul * self.w - T::one();
        let x_val = v.x;
        let y_val = v.y;
        let z_val = v.z;

        TVec3 {
            x: vmul * x_val + dotp * self.x + cross_mul * (self.y * z_val - self.z * y_val),
            y: vmul * y_val + dotp * self.y + cross_mul * (self.z * x_val - self.x * z_val),
            z: vmul * z_val + dotp * self.z + cross_mul * (self.x * y_val - self.y * x_val),
        }
    }
}

pub fn to_rotation_matrix<T: Float>(q: Quat<T>) -> Mat4<T> {
    let two: T = T::one() + T::one();

    let s = two / length_squared(q);

    let xs = s * q.x;
    let ys = s * q.y;
    let zs = s * q.z;

    let wx = q.w * xs;
    let wy = q.w * ys;
    let wz = q.w * zs;

    let xx = q.x * xs;
    let xy = q.x * ys;
    let xz = q.x * zs;

    let yy = q.y * ys;
    let yz = q.y * zs;

    let zz = q.z * zs;

    Mat4 {
        //
        // 1st row
        a00: T::one() - (yy + zz),
        a01: xy - wz,
        a02: xz + wy,
        a03: T::zero(),
        //
        // 2nd row
        a10: xy + wz,
        a11: T::one() - (xx + zz),
        a12: yz - wx,
        a13: T::zero(),
        //
        // 3rd row
        a20: xz - wy,
        a21: yz + wx,
        a22: T::one() - (xx + yy),
        a23: T::zero(),
        //
        // 4th row
        a30: T::zero(),
        a31: T::zero(),
        a32: T::zero(),
        a33: T::one(),
    }
}

pub fn conjugate<T>(q: Quat<T>) -> Quat<T>
where
    T: Copy + Clone + Float,
{
    Quat {
        w: q.w,
        x: -q.x,
        y: -q.y,
        z: -q.z,
    }
}

pub fn length_squared<T>(q: Quat<T>) -> T
where
    T: Copy + Clone + Float,
{
    q.w * q.w + q.x * q.x + q.y * q.y + q.z * q.z
}

pub fn length<T>(q: Quat<T>) -> T
where
    T: Copy + Clone + Float,
{
    length_squared(q).sqrt()
}

pub fn normalize<T>(q: Quat<T>) -> Quat<T>
where
    T: Float + std::fmt::Debug,
{
    let len_squared = length_squared(q);
    if len_squared.is_zero() {
        self::consts::identity()
    } else {
        let k = len_squared.recip();
        Quat {
            w: k * q.w,
            x: k * q.x,
            y: k * q.y,
            z: k * q.z,
        }
    }
}

pub fn invert<T>(q: Quat<T>) -> Quat<T>
where
    T: Float + std::fmt::Debug,
{
    let len_squared = length_squared(q);
    if len_squared.is_zero() {
        self::consts::identity()
    } else {
        let k = len_squared.recip();
        Quat {
            w: k * q.w,
            x: -k * q.x,
            y: -k * q.y,
            z: -k * q.z,
        }
    }
}

pub fn dot<T>(a: Quat<T>, b: Quat<T>) -> T
where
    T: Float,
{
    a.w * b.w + a.x * b.x + a.y * b.y + a.z * b.z
}

pub fn is_unit_length<T: Float>(q: Quat<T>) -> bool {
    length_squared(q).is_one()
}

pub fn is_zero<T: Float>(q: Quat<T>) -> bool {
    length_squared(q).is_zero()
}
