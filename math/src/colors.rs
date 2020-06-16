use crate::utility::saturate;
use num_traits::Num;

pub trait NumColorComponent<ComponentType = Self> {
    fn alpha_max() -> ComponentType;
    fn from_u32(val: u32) -> (ComponentType, ComponentType, ComponentType, ComponentType);
    fn to_u32(r: ComponentType, g: ComponentType, b: ComponentType, a: ComponentType) -> u32;
}

impl NumColorComponent for u8 {
    fn alpha_max() -> u8 {
        255
    }

    fn from_u32(c: u32) -> (u8, u8, u8, u8) {
        (
            ((c >> 16) & 0xFF) as u8,
            ((c >> 8) & 0xFF) as u8,
            (c & 0xFF) as u8,
            ((c >> 24) & 0xFF) as u8,
        )
    }

    fn to_u32(r: u8, g: u8, b: u8, a: u8) -> u32 {
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }
}

impl NumColorComponent for f32 {
    fn alpha_max() -> f32 {
        1f32
    }

    fn from_u32(c: u32) -> (f32, f32, f32, f32) {
        (
            ((c >> 16) & 0xFF) as f32 / 255_f32,
            ((c >> 8) & 0xFF) as f32 / 255_f32,
            (c & 0xFF) as f32 / 255_f32,
            ((c >> 24) & 0xFF) as f32 / 255_f32,
        )
    }

    fn to_u32(r: f32, g: f32, b: f32, a: f32) -> u32 {
        let r = (saturate(r) * 255_f32) as u32;
        let g = (saturate(g) * 255_f32) as u32;
        let b = (saturate(b) * 255_f32) as u32;
        let a = (saturate(a) * 255_f32) as u32;

        (a << 24) | (r << 16) | (g << 8) | b
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T> TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
    pub fn new(r: T, g: T, b: T) -> Self {
        Self::new_with_alpha(r, g, b, T::alpha_max())
    }

    pub fn new_with_alpha(r: T, g: T, b: T, a: T) -> Self {
        TColorRGBA { r, g, b, a }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self as *const TColorRGBA<T> as *const T, 4) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self as *mut Self as *mut T, 4) }
    }

    pub fn as_ptr(&self) -> *const T {
        &self.r as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        &mut self.r as *mut _
    }
}

pub type RGBAColor = TColorRGBA<u8>;
pub type RGBAColorF32 = TColorRGBA<f32>;

impl<T> std::default::Default for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
    fn default() -> Self {
        Self::new_with_alpha(T::zero(), T::zero(), T::zero(), T::zero())
    }
}

impl<T> std::convert::From<(T, T, T, T)> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
    fn from(c: (T, T, T, T)) -> Self {
        Self::new_with_alpha(c.0, c.1, c.2, c.3)
    }
}

impl<T> std::convert::From<(T, T, T)> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
    fn from(c: (T, T, T)) -> Self {
        Self::new_with_alpha(c.0, c.1, c.2, T::alpha_max())
    }
}

impl<T> std::convert::From<[T; 4]> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
    fn from(c: [T; 4]) -> Self {
        Self::new_with_alpha(c[0], c[1], c[2], c[3])
    }
}

impl<T> std::convert::From<[T; 3]> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent,
{
    fn from(c: [T; 3]) -> Self {
        Self::new_with_alpha(c[0], c[1], c[2], T::alpha_max())
    }
}

impl<T> std::convert::From<u32> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    fn from(val: u32) -> Self {
        T::from_u32(val).into()
    }
}

impl<T> std::convert::From<TColorRGBA<T>> for u32
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    fn from(c: TColorRGBA<T>) -> u32 {
        T::to_u32(c.r, c.g, c.b, c.a)
    }
}

impl std::convert::From<RGBAColor> for RGBAColorF32 {
    fn from(rgba: RGBAColor) -> Self {
        RGBAColorF32::new_with_alpha(
            rgba.r as f32 / 255_f32,
            rgba.g as f32 / 255_f32,
            rgba.b as f32 / 255_f32,
            rgba.a as f32 / 255_f32,
        )
    }
}

impl std::convert::From<RGBAColorF32> for RGBAColor {
    fn from(rgbaf32: RGBAColorF32) -> Self {
        RGBAColor::new_with_alpha(
            (saturate(rgbaf32.r) * 255_f32) as u8,
            (saturate(rgbaf32.g) * 255_f32) as u8,
            (saturate(rgbaf32.b) * 255_f32) as u8,
            (saturate(rgbaf32.a) * 255_f32) as u8,
        )
    }
}

impl std::convert::From<(u8, u8, u8, u8)> for RGBAColorF32 {
    fn from(c: (u8, u8, u8, u8)) -> Self {
        Self::new_with_alpha(
            c.0 as f32 / 255f32,
            c.1 as f32 / 255 as f32,
            c.2 as f32 / 255f32,
            c.3 as f32 / 255f32,
        )
    }
}

impl std::convert::From<(u8, u8, u8)> for RGBAColorF32 {
    fn from(c: (u8, u8, u8)) -> Self {
        Self::new_with_alpha(
            c.0 as f32 / 255f32,
            c.1 as f32 / 255 as f32,
            c.2 as f32 / 255f32,
            1f32,
        )
    }
}

impl std::convert::From<[u8; 4]> for RGBAColorF32 {
    fn from(c: [u8; 4]) -> RGBAColorF32 {
        Self::new_with_alpha(
            c[0] as f32 / 255f32,
            c[1] as f32 / 255f32,
            c[2] as f32 / 255f32,
            c[3] as f32 / 255f32,
        )
    }
}

impl std::convert::From<[u8; 3]> for RGBAColorF32 {
    fn from(c: [u8; 3]) -> RGBAColorF32 {
        Self::new_with_alpha(
            c[0] as f32 / 255f32,
            c[1] as f32 / 255f32,
            c[2] as f32 / 255f32,
            1f32,
        )
    }
}

impl<T> std::convert::From<&str> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    fn from(s: &str) -> Self {
        let s = s.trim();
        if s.is_empty() {
            return TColorRGBA::new_with_alpha(T::zero(), T::zero(), T::zero(), T::zero());
        }

        let s = if s.starts_with('#') { &s[1..] } else { s };

        let len_content = s.len();
        if !(len_content == 6 || len_content == 8) {
            return TColorRGBA::new_with_alpha(T::zero(), T::zero(), T::zero(), T::zero());
        }

        u32::from_str_radix(s, 16).map_or_else(
            |_| TColorRGBA::new_with_alpha(T::zero(), T::zero(), T::zero(), T::zero()),
            |color_u32| {
                let color_u32 = if len_content == 6 {
                    color_u32 | (0xFF << 24)
                } else {
                    color_u32
                };

                Self::from(color_u32)
            },
        )
    }
}

impl<T> std::ops::Deref for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> std::ops::DerefMut for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T> std::convert::AsRef<[T]> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> std::convert::AsMut<[T]> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> std::borrow::Borrow<[T]> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> std::borrow::BorrowMut<[T]> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> std::fmt::Display for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Display + std::fmt::Debug + Num + NumColorComponent,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[r:{}, g:{}, b:{}, a:{}]",
            self.r, self.g, self.b, self.a
        )
    }
}

impl<T> std::ops::AddAssign for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.as_mut_slice()
            .iter_mut()
            .zip(rhs.as_slice().iter())
            .for_each(|(s, r)| *s += *r);
    }
}

impl<T> std::ops::SubAssign for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.as_mut_slice()
            .iter_mut()
            .zip(rhs.as_slice().iter())
            .for_each(|(s, r)| *s -= *r);
    }
}

impl<T> std::ops::MulAssign for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::MulAssign,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.as_mut_slice()
            .iter_mut()
            .zip(rhs.as_slice().iter())
            .for_each(|(s, r)| *s *= *r);
    }
}

impl<T> std::ops::MulAssign<T> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::MulAssign,
{
    fn mul_assign(&mut self, k: T) {
        self.as_mut_slice().iter_mut().for_each(|s| *s *= k);
    }
}

impl<T> std::ops::DivAssign for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::DivAssign,
{
    fn div_assign(&mut self, rhs: Self) {
        self.as_mut_slice()
            .iter_mut()
            .zip(rhs.as_slice().iter())
            .for_each(|(s, r)| *s /= *r);
    }
}

impl<T> std::ops::DivAssign<T> for TColorRGBA<T>
where
    T: Copy + Clone + std::fmt::Debug + Num + NumColorComponent + std::ops::DivAssign,
{
    fn div_assign(&mut self, k: T) {
        self.as_mut_slice().iter_mut().for_each(|s| *s /= k);
    }
}

impl<T> std::ops::Add for TColorRGBA<T>
where
    T: Copy
        + Clone
        + std::fmt::Debug
        + Num
        + NumColorComponent
        + std::ops::Add<Output = T>
        + std::ops::AddAssign,
{
    type Output = Self;

    fn add(self, rhs: TColorRGBA<T>) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl<T> std::ops::Sub for TColorRGBA<T>
where
    T: Copy
        + Clone
        + std::fmt::Debug
        + Num
        + NumColorComponent
        + std::ops::Sub<Output = T>
        + std::ops::SubAssign,
{
    type Output = Self;

    fn sub(self, rhs: TColorRGBA<T>) -> Self::Output {
        let mut result = self;
        result -= rhs;
        result
    }
}

impl<T> std::ops::Mul for TColorRGBA<T>
where
    T: Copy
        + Clone
        + std::fmt::Debug
        + Num
        + NumColorComponent
        + std::ops::Mul<Output = T>
        + std::ops::MulAssign,
{
    type Output = Self;

    fn mul(self, rhs: TColorRGBA<T>) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl<T> std::ops::Mul<T> for TColorRGBA<T>
where
    T: Copy
        + Clone
        + std::fmt::Debug
        + Num
        + NumColorComponent
        + std::ops::Mul<Output = T>
        + std::ops::MulAssign,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl<T> std::ops::Div for TColorRGBA<T>
where
    T: Copy
        + Clone
        + std::fmt::Debug
        + Num
        + NumColorComponent
        + std::ops::Div<Output = T>
        + std::ops::DivAssign,
{
    type Output = Self;

    fn div(self, rhs: TColorRGBA<T>) -> Self::Output {
        let mut result = self;
        result /= rhs;
        result
    }
}

impl<T> std::ops::Div<T> for TColorRGBA<T>
where
    T: Copy
        + Clone
        + std::fmt::Debug
        + Num
        + NumColorComponent
        + std::ops::Div<Output = T>
        + std::ops::DivAssign,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let mut result = self;
        result /= rhs;
        result
    }
}

macro_rules! define_color_type {
    ( $classname:ident, $fieldstype:ty, $numfields:expr, $( ($membername:ident => $initname:ident) ),+ ) => {
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $classname {
            $(
                pub $membername : $fieldstype
            ),+
        }

        impl $classname {
            pub fn new( $($initname : $fieldstype),+ ) -> Self {
                Self {
                    $(
                        $membername : $initname
                    ),+
                }
            }

            pub fn as_slice(&self) -> &[$fieldstype] {
                unsafe {
                    std::slice::from_raw_parts(self as *const _ as *const $fieldstype, $numfields)
                }
            }

            pub fn as_mut_slice(&mut self) -> &mut [$fieldstype] {
                unsafe {
                    std::slice::from_raw_parts_mut(self as *mut _ as *mut $fieldstype, $numfields)
                }
            }
        }

        impl std::ops::Deref for $classname {
            type Target = [$fieldstype];
            fn deref(&self) -> &Self::Target {
                self.as_slice()
            }
        }

        impl std::ops::DerefMut for $classname {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut_slice()
            }
        }

        impl std::convert::AsRef<[$fieldstype]> for $classname {
            fn as_ref(&self) -> &[$fieldstype] {
                self.as_slice()
            }
        }

        impl std::convert::AsMut<[$fieldstype]> for $classname {
            fn as_mut(&mut self) -> &mut [$fieldstype] {
                self.as_mut_slice()
            }
        }

        impl std::borrow::Borrow<[$fieldstype]> for $classname {
            fn borrow(&self) -> &[$fieldstype] {
                self.as_slice()
            }
        }

        impl std::borrow::BorrowMut<[$fieldstype]> for $classname {
            fn borrow_mut(&mut self) -> &mut [$fieldstype] {
                self.as_mut_slice()
            }
        }
    };
}

define_color_type!(HsvColor, f32, 3usize, (h => hue), (s => saturation), (v => value));
define_color_type!(HslColor, f32, 3usize, (h => hue), (s => lightness), (l => saturation));
define_color_type!(XyzColor, f32, 3usize, (x => xval), (y => yval), (z => zval));

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::From;

    #[test]
    fn test_conversion_from_html() {
        assert_eq!(
            RGBAColor::from("eb4034"),
            RGBAColor::new_with_alpha(235, 64, 52, 255)
        );

        assert_eq!(
            RGBAColor::from("#eb4034"),
            RGBAColor::new_with_alpha(235, 64, 52, 255)
        );

        assert_eq!(RGBAColor::from("#ffeb4034"), RGBAColor::from("eb4034"));

        assert_eq!(
            RGBAColor::from("ff34eb9e"),
            RGBAColor::new_with_alpha(52, 235, 158, 255)
        );

        assert_eq!(
            RGBAColor::from("#ff34eb9e"),
            RGBAColor::new_with_alpha(52, 235, 158, 255)
        );

        assert_eq!(
            RGBAColor::from("jdk4fxuu"),
            RGBAColor::new_with_alpha(0, 0, 0, 0)
        );
    }

    #[test]
    fn test_conversion_from_slice() {
        let clr = [52u8, 235u8, 158u8, 255u8];
        assert_eq!(
            RGBAColor::from(clr),
            RGBAColor::new_with_alpha(52, 235, 158, 255)
        );

        let c: u32 = RGBAColor::new(52, 235, 158).into();
        assert_eq!(c, 0xff34eb9e);
    }
}
