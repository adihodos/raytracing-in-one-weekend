use crate::hittable::HitRecord;
use crate::types::{Color, Ray};

#[derive(Copy, Clone, Debug)]
pub struct ScatterRecord {
    pub ray: Ray,
    pub attenuation: Color,
}

pub trait Material {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord>;
}
