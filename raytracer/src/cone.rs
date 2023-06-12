use std::sync::Arc;

use math::polynomial::poly_quadratic;

use crate::{
    aabb3::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    types::{Real, C_ONE, C_TWO, C_TWO_PI, C_ZERO},
};

pub struct Cone {
    radius: Real,
    height: Real,
    phi_max: Real,
    mtl: Arc<dyn Material>,
}

impl Cone {
    pub fn new(radius: Real, height: Real, phi_max: Real, mtl: Arc<dyn Material>) -> Cone {
        Cone {
            radius,
            height,
            phi_max,
            mtl,
        }
    }

    pub fn unit(phi_max: Option<Real>, mtl: Arc<dyn Material>) -> Cone {
        Self::new(C_ONE, C_ONE, phi_max.unwrap_or_else(|| C_TWO_PI), mtl)
    }
}

impl Hittable for Cone {
    fn hit(
        &self,
        r: &crate::types::Ray,
        _t_min: Real,
        t_max: Real,
    ) -> Option<crate::hittable::HitRecord> {
        let k = self.radius / self.height;
        let k = k * k;

        let a = r.direction.x * r.direction.x + r.direction.y * r.direction.y
            - k * r.direction.z * r.direction.z;
        let b = C_TWO
            * (r.direction.x * r.origin.x + r.direction.y * r.origin.y
                - k * r.direction.z * (r.origin.z - self.height));
        let c = r.origin.x * r.origin.x + r.origin.y * r.origin.y
            - k * (r.origin.z - self.height) * (r.origin.z - self.height);

        let mut roots = [C_ZERO; 2];
        if poly_quadratic(a, b, c, &mut roots) == 0 {
            return None;
        }

        let [t0, t1] = roots;

        if t0 > t_max || t1 <= C_ZERO {
            return None;
        }

        let mut thit = if t0 <= C_ZERO { t1 } else { t0 };
        if thit > t_max {
            return None;
        }

        let mut p = r.at(thit);
        let mut phi = p.y.atan2(p.x);

        if phi < C_ZERO {
            phi += C_TWO_PI;
        }

        //
        // test cone intersection against clipping parameters
        if p.z < C_ZERO || p.z > self.height || phi > self.phi_max {
            if thit == t1 {
                return None;
            }

            thit = t1;

            if thit > t_max {
                return None;
            }

            p = r.at(thit);
            phi = p.y.atan2(p.x);

            if phi < C_ZERO {
                phi += C_TWO_PI;
            }

            if p.z < C_ZERO || p.z > self.height || phi > self.phi_max {
                return None;
            }
        }

        //
        // find parametric representation of cone hit
        let u = phi / self.phi_max;
        let v = p.z / self.height;

        use crate::types::Vec3;
        use math::vec3;
        let dpdu = Vec3::new(-self.phi_max * p.y, self.phi_max * p.y, C_ZERO);
        let dpdv = Vec3::new(-p.x / (C_ONE - v), -p.y / (C_ONE - v), self.height);

        Some(HitRecord::new(
            p,
            vec3::normalize(vec3::cross(dpdu, dpdv)),
            r,
            thit,
            self.mtl.clone(),
            u,
            v,
        ))
    }

    fn bounding_box(&self, _time0: Real, _time1: Real) -> Option<Aabb> {
        Some(Aabb {
            min: (-self.radius, -self.radius, C_ZERO).into(),
            max: (self.radius, self.radius, self.height).into(),
        })
    }
}
