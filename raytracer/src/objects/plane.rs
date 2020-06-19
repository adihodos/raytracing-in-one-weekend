use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::types::{Point, Ray, Real, Vec3};

#[derive(Clone)]
pub struct Plane {
    pub origin: Point,
    pub normal: Vec3,
    pub mtl: std::sync::Arc<dyn Material>,
}

impl Hittable for Plane {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        use math::vec3::dot;

        let dir_dot_normal = dot(self.normal, r.direction);
        const EPSILON: Real = 1.0E-5 as Real;

        if dir_dot_normal.abs() < EPSILON {
            //
            // ray is parallel or contained in the plane
            return None;
        }

        let a = r.origin - self.origin;
        let t = dot(a, self.normal) / dir_dot_normal;

        if t < t_max && t > t_min {
            Some(HitRecord::new(
                r.at(t),
                self.normal,
                r,
                t,
                std::sync::Arc::clone(&self.mtl),
            ))
        } else {
            None
        }
    }
}
