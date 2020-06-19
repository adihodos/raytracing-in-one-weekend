use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::types::{Point, Ray, Real, Vec3};

#[derive(Clone)]
pub struct Plane {
    pub normal: Vec3,
    pub d: Real,
    pub mtl: std::sync::Arc<dyn Material>,
}

impl Plane {
    pub fn new(origin: Point, normal: Vec3, mtl: std::sync::Arc<dyn Material>) -> Plane {
        Plane {
            normal,
            d: math::vec3::dot(normal, origin),
            mtl,
        }
    }
}

impl Hittable for Plane {
    fn hit(&self, ray: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        use math::vec3::dot;

        //
        // plane with normal N and D scalar
        // ray with origin A and direction B

        const EPSILON: Real = 1.0E-5 as Real;
        let b_dot_n = dot(ray.direction, self.normal);

        if b_dot_n.abs() < EPSILON {
            //
            // ray is parallel or contained in the plane
            return None;
        }

        let a_dot_n = dot(ray.origin, self.normal);
        let temp = (self.d - a_dot_n) / b_dot_n;

        if temp < t_max && temp > t_min {
            //
            // intersection point is on the ray
            Some(HitRecord::new(
                ray.at(temp),
                self.normal,
                ray,
                temp,
                std::sync::Arc::clone(&self.mtl),
            ))
        } else {
            //
            // intersection point is behind the ray
            None
        }
    }
}
