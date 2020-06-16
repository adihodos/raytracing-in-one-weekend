use crate::hittable::{HitRecord, Hittable};
use crate::types::{Point, Ray, Real};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sphere {
    pub center: Point,
    pub radius: Real,
}

impl Sphere {
    pub fn new(center: Point, radius: Real) -> Sphere {
        Sphere { center, radius }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        use math::vec3::dot;

        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = dot(oc, r.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let delta = half_b * half_b - a * c;

        if delta > 0f32 {
            let root = delta.sqrt();
            let temp = (-half_b - root) / a;
            if temp < t_max && temp > t_min {
                let p = r.at(temp);
                Some(HitRecord {
                    t: temp,
                    p,
                    normal: (p - self.center) / self.radius,
                })
            } else {
                let temp = (-half_b + root) / a;
                if temp < t_max && temp > t_min {
                    let p = r.at(temp);
                    Some(HitRecord {
                        t: temp,
                        p,
                        normal: (p - self.center) / self.radius,
                    })
                } else {
                    None
                }
            }
        } else {
            None
        }
    }
}
