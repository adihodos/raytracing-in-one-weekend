use crate::types::{Color, Point, Real};

pub trait Texture: Send + Sync {
    fn value(&self, u: Real, v: Real, point: Point) -> Color;
}
