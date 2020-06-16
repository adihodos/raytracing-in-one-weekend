#![allow(dead_code)]

pub fn saturate(x: f32) -> f32 {
    (x.min(1_f32)).max(0_f32)
}

pub fn clamp(minval: f32, x: f32, maxval: f32) -> f32 {
    minval.max(x).min(maxval)
}

pub fn roundup_next_power_of_two(x: u32) -> u32 {
    if x == 0 {
        x
    } else {
        let mut x = x;
        x -= 1;
        x |= x >> 1;
        x |= x >> 2;
        x |= x >> 4;
        x |= x >> 8;
        x |= x >> 16;
        x += 1;

        x
    }
}

pub fn roundup_multiple_of(x: u32, target: u32) -> u32 {
    ((x / target) + 1) * target
}

pub fn size_of_slice<T>(s: &[T]) -> usize {
    std::mem::size_of::<T>() * s.len()
}

#[macro_export]
macro_rules! gen_multiply_vector_to_scalar_ops {
    ($vec:ty, $t:ty) => {
        impl std::ops::Mul<$vec> for $t
        where
            $t: Copy + Clone + std::fmt::Debug + num::Num,
        {
            type Output = $vec;

            fn mul(self, rhs: $vec) -> Self::Output {
                rhs * self
            }
        }
    };
}
