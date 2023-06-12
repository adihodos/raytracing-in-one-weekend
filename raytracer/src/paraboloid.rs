use std::sync::Arc;

use crate::{
    aabb3::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    types::{Ray, Real, C_HALF_ONE, C_ONE, C_TWO, C_TWO_PI, C_ZERO},
};

pub struct Paraboloid {
    radius: Real,
    zmin: Real,
    zmax: Real,
    phi_max: Real,
    mtl: Arc<dyn Material>,
}

impl Paraboloid {
    pub fn unit(mtl: Arc<dyn Material>) -> Self {
        Self::new(C_ONE, -C_HALF_ONE, C_HALF_ONE, C_TWO_PI, mtl)
    }

    pub fn new(
        radius: Real,
        zmin: Real,
        zmax: Real,
        phi_max: Real,
        mtl: Arc<dyn Material>,
    ) -> Self {
        Self {
            radius,
            zmin: zmin.min(zmax),
            zmax: zmin.max(zmax),
            phi_max,
            mtl,
        }
    }
}

impl Hittable for Paraboloid {
    fn bounding_box(&self, _time0: Real, _time11: Real) -> Option<Aabb> {
        Some(Aabb {
            min: (-self.radius, -self.radius, self.zmin).into(),
            max: (self.radius, self.radius, self.zmax).into(),
        })
    }

    fn hit(&self, r: &Ray, _t_min: Real, t_max: Real) -> Option<HitRecord> {
        //
        // Physically based rendering, section 3.5.2, page 134
        let dx = r.direction.x;
        let dy = r.direction.y;
        let dz = r.direction.z;

        let ox = r.origin.x;
        let oy = r.origin.y;
        let oz = r.origin.z;

        let k = self.zmax / (self.radius * self.radius);

        let a = k * (dx * dx + dy * dy);
        let b = C_TWO * k * (dx * ox + dy * oy) - dz;
        let c = k * (ox * ox + oy * oy) - oz;

        //
        // Solve quadratic equation for _t_ values

        let mut roots = [C_ZERO; 2];
        if math::polynomial::poly_quadratic(a, b, c, &mut roots) == 0 {
            return None;
        }

        let [t0, t1] = roots;

        //
        // Check quadric shape _t0_ and _t1_ for nearest intersection

        if t0 > t_max || t1 <= C_ZERO {
            return None;
        }

        let mut thit = t0;
        if thit <= C_ZERO {
            thit = t1;
            if thit > t_max {
                return None;
            }
        }

        //
        // Compute paraboloid inverse mapping
        let mut phit = r.at(thit);
        let mut phi = phit.y.atan2(phit.x);
        if phi < C_ZERO {
            phi += C_TWO_PI;
        }

        //
        // Test paraboloid intersection against clipping parameters
        if phit.z < self.zmin || phit.z > self.zmax || phi > self.phi_max {
            if thit == t1 {
                return None;
            }
            thit = t1;
            if t1 > t_max {
                return None;
            }
            //
            // Compute paraboloid inverse mapping
            phit = r.at(thit);
            phi = phit.y.atan2(phit.x);
            if phi < C_ZERO {
                phi += C_TWO_PI;
            }

            if phit.z < self.zmin || phit.z > self.zmax || phi > self.phi_max {
                return None;
            }
        }

        //
        // Find parametric representation of paraboloid hit
        let u = phi / self.phi_max;
        let v = (phit.z - self.zmin) / (self.zmax - self.zmin);

        //
        // Compute paraboloid dpdu$ and dpdv$

        use crate::types::Vec3;
        let dpdu = Vec3::new(-self.phi_max * phit.y, self.phi_max * phit.x, C_ZERO);
        let dpdv = (self.zmax - self.zmin)
            * Vec3::new(phit.x / (C_TWO * phit.z), phit.y / (C_TWO * phit.z), C_ONE);

        use math::vec3::{cross, normalize};
        Some(HitRecord::new(
            phit,
            normalize(cross(dpdu, dpdv)),
            r,
            thit,
            self.mtl.clone(),
            u,
            v,
        ))
    }
}
