use crate::types::{Point, Ray, Real, Vec3};

#[derive(Copy, Clone, Debug)]
pub struct HitRecord {
    pub p: Point,
    pub normal: Vec3,
    pub t: Real,
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord>;
}
