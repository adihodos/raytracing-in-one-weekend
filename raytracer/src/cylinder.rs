use std::sync::Arc;

use math::polynomial::poly_quadratic;

use crate::{
    aabb3::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    types::{Ray, Real, Vec3, C_HALF_ONE, C_INFINITY, C_ONE, C_PI, C_TWO, C_TWO_PI, C_ZERO},
};

pub struct Cylinder {
    radius: Real,
    zmin: Real,
    zmax: Real,
    phi_max: Real,
    aabb: Aabb,
    mtl: Arc<dyn Material>,
}

impl Cylinder {
    pub fn unit(mtl: Arc<dyn Material>) -> Self {
        Self::new(C_ONE, -C_HALF_ONE, C_HALF_ONE, C_TWO_PI, mtl)
    }

    pub fn new(radius: Real, z0: Real, z1: Real, phi_max: Real, mtl: Arc<dyn Material>) -> Self {
        let zmin = z0.min(z1);
        let zmax = z0.max(z1);

        Cylinder {
            radius,
            zmin,
            zmax,
            phi_max,
            aabb: Aabb::new((-radius, -radius, zmin), (radius, radius, zmax)),
            mtl,
        }
    }

    fn area(&self) -> Real {
        (self.zmax - self.zmin) * self.radius * self.phi_max
    }
}

impl Hittable for Cylinder {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<crate::hittable::HitRecord> {
        //
        // compute quadratic coefficients
        let a = r.direction.x * r.direction.x + r.direction.y * r.direction.y;
        let b = C_TWO * (r.direction.x * r.origin.x + r.direction.y * r.origin.y);
        let c = r.origin.x * r.origin.x + r.origin.y * r.origin.y - self.radius * self.radius;

        let mut roots: [Real; 2] = [C_ZERO; 2];
        let num_roots = poly_quadratic(a, b, c, &mut roots);

        if num_roots == 0 {
            return None;
        }

        let [t0, t1] = roots;

        if t0 > t_max || t1 < t_min {
            return None;
        }

        let mut thit = if t0 < t_min { t1 } else { t0 };

        if thit > t_max {
            return None;
        }

        let mut p = r.at(thit);
        let mut phi = p.y.atan2(p.x);
        phi = if phi < C_ZERO {
            phi + C_TWO * C_PI
        } else {
            phi
        };

        //
        // test intersection against clipping parameters
        if p.z < self.zmin || p.z > self.zmax || phi > self.phi_max {
            if thit == t1 {
                return None;
            }

            thit = t1;

            if t1 > t_max {
                return None;
            }

            p = r.at(thit);
            phi = p.y.atan2(p.x);
            phi = if phi < C_ZERO {
                phi + C_TWO * C_PI
            } else {
                phi
            };

            if p.z < self.zmin || p.z > self.zmax || phi > self.phi_max {
                return None;
            }
        }

        let u = phi / self.phi_max;
        let v = (p.z - self.zmin) / (self.zmax - self.zmin);

        let dpdu = Vec3::new(-self.phi_max * p.y, self.phi_max * p.x, C_ZERO);
        let dpdv = Vec3::new(C_ZERO, C_ZERO, self.zmax - self.zmin);

        Some(HitRecord::new(
            p,
            // math::vec3::normalize(math::vec3::cross(dpdu, dpdv)),
            math::vec3::normalize(math::vec3::cross(dpdv, dpdu)),
            r,
            thit,
            self.mtl.clone(),
            u,
            v,
        ))
    }

    fn bounding_box(&self, _time0: Real, _time1: Real) -> Option<Aabb> {
        Some(self.aabb)
    }

    fn pdf_value(&self, o: crate::types::Point, v: Vec3) -> Real {
        self.hit(&Ray::new(o, v, C_ZERO), 0.0001 as Real, C_INFINITY)
            .map_or_else(
                || C_ZERO,
                |hit| {
                    use math::vec3::{dot, length_squared};
                    let pdf = (C_ONE / self.area())
                        / (dot(hit.normal, -v).abs() / length_squared(o - hit.p));

                    if pdf.is_infinite() {
                        C_ZERO
                    } else {
                        pdf
                    }
                },
            )
    }

    fn random(&self, v: Vec3) -> Vec3 {
        let direction = self.aabb.center() - v;
        use math::vec3::length_squared;
        let distance_squared = length_squared(direction);
        let uvw: crate::onb::Onb = direction.into();
        uvw.local_from_vec(crate::types::random_to_sphere(
            self.radius,
            distance_squared,
        ))
    }
}
