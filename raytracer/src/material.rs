use crate::hittable::HitRecord;
use crate::types::{Color, Point, Ray, Real};

#[derive(Copy, Clone, Debug)]
pub struct ScatterRecord {
    pub ray: Ray,
    pub attenuation: Color,
}

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord>;

    fn emitted(&self, _u: Real, _v: Real, _point: Point) -> Color {
        Color::broadcast(0 as Real)
    }
}
