use crate::material::Material;
use crate::types::{Point, Ray, Real, Vec3};

#[derive(Clone)]
pub struct HitRecord {
    pub p: Point,
    pub normal: Vec3,
    pub t: Real,
    pub mtl: std::sync::Arc<dyn Material>,
    pub front_face: bool,
    pub u: Real,
    pub v: Real,
}

impl HitRecord {
    pub fn new(
        p: Point,
        outward_normal: Vec3,
        ray: &Ray,
        t: Real,
        mtl: std::sync::Arc<dyn Material>,
        u: Real,
        v: Real,
    ) -> HitRecord {
        let front_face = !math::vec3::are_on_the_same_plane_side(ray.direction, outward_normal);
        HitRecord {
            p,
            normal: if front_face {
                outward_normal
            } else {
                -outward_normal
            },
            t,
            mtl: std::sync::Arc::clone(&mtl),
            front_face,
            u,
            v,
        }
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord>;
}
