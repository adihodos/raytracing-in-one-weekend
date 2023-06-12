use std::sync::Arc;

use math::polynomial::poly_quadratic;
use num::Zero;

use crate::{
    aabb3::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    types::{self, Real, Vec3, C_ONE, C_TWO, C_TWO_PI, C_ZERO},
};

pub struct Hyperboloid {
    p1: Vec3,
    p2: Vec3,
    zmin: Real,
    zmax: Real,
    phi_max: Real,
    rmax: Real,
    ah: Real,
    ch: Real,
    mtl: Arc<dyn Material>,
}

impl Hyperboloid {
    pub fn new(p1: Vec3, p2: Vec3, phi_max: Real, mtl: Arc<dyn Material>) -> Hyperboloid {
        let radius1 = (p1.x * p1.x + p1.y * p1.y).sqrt();
        let radius2 = (p2.x * p2.x + p2.y * p2.y).sqrt();
        let rmax = radius1.max(radius2);
        let zmin = p1.z.min(p2.z);
        let zmax = p1.z.max(p2.z);
        //
        // Compute implicit function coefficients for hyperboloid
        let (p1, p2) = if p2.z.is_zero() { (p2, p1) } else { (p1, p2) };

        let mut pp = p1;
        let mut xy1 = C_ZERO;
        let mut xy2 = C_ZERO;
        let mut ah = C_ZERO;
        let mut ch = C_ZERO;

        loop {
            pp += C_TWO * (p2 - p1);
            xy1 = pp.x * pp.x + pp.y * pp.y;
            xy2 = p2.x * p2.x + p2.y * p2.y;

            ah = (xy1.recip() - (pp.z * pp.z) / (xy1 * p2.z * p2.z))
                / (C_ONE - (xy2 * pp.z * pp.z) / (xy1 * p2.z * p2.z));
            ch = (ah * xy2 - C_ONE) / (p2.z * p2.z);

            if !ah.is_infinite() && !ah.is_nan() {
                break;
            }
        }

        Self {
            p1,
            p2,
            zmin,
            zmax,
            phi_max,
            rmax,
            ah,
            ch,
            mtl,
        }
    }
}

impl Hittable for Hyperboloid {
    fn bounding_box(&self, _time0: Real, _time1: Real) -> Option<Aabb> {
        Some(Aabb {
            min: (-self.rmax, -self.rmax, self.zmin).into(),
            max: (self.rmax, self.rmax, self.zmax).into(),
        })
    }

    fn hit(&self, r: &types::Ray, _t_min: Real, t_max: Real) -> Option<HitRecord> {
        //
        // Physically based rendering, page 134
        let (ox, oy, oz) = (r.origin.x, r.origin.y, r.origin.z);
        let (dx, dy, dz) = (r.direction.x, r.direction.y, r.direction.z);

        let a = self.ah * dx * dx + self.ah * dy * dy - self.ch * dz * dz;
        let b = C_TWO * (self.ah * dx * ox + self.ah * dy * oy - self.ch * dz * oz);
        let c = self.ah * ox * ox + self.ah * oy * oy - self.ch * oz * oz - C_ONE;

        let mut roots = [C_ZERO; 2];
        if poly_quadratic(a, b, c, &mut roots) == 0 {
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
        // Compute hyperboloid inverse mapping
        let mut phit = r.at(thit);
        let mut v = (phit.z - self.p1.z) / (self.p2.z - self.p1.z);
        let pr = (C_ONE - v) * self.p1 + v * self.p2;
        let mut phi = (pr.x * phit.y - phit.x * pr.y).atan2(phit.x * pr.x + phit.y * pr.y);
        if phi < C_ZERO {
            phi += C_TWO;
        }

        //
        // Test hyperboloid intersection against clipping parameters
        if phit.z < self.zmin || phit.z > self.zmax || phi > self.phi_max {
            if thit == t1 {
                return None;
            }

            thit = t1;
            if t1 > t_max {
                return None;
            }

            //
            // Compute hyperboloid inverse mapping
            phit = r.at(thit);
            v = (phit.z - self.p1.z) / (self.p2.z - self.p1.z);
            let pr = (C_ONE - v) * self.p1 + v * self.p2;
            phi = (pr.x * phit.y - phit.x * pr.y).atan2(phit.x * pr.x + phit.y * pr.y);
            if phi < C_ZERO {
                phi += C_TWO_PI;
            }

            if phit.z < self.zmin || phit.z > self.zmax || phi > self.phi_max {
                return None;
            }
        }

        //
        // Compute parametric representation of hyperboloid hit
        let u = phi / self.phi_max;

        // Compute hyperboloid $\dpdu$ and $\dpdv$
        let (sin_phi, cos_phi) = phi.sin_cos();
        let dpdu = Vec3::new(-self.phi_max * phit.y, self.phi_max * phit.x, C_ZERO);
        let dpdv = Vec3::new(
            (self.p2.x - self.p1.x) * cos_phi - (self.p2.y - self.p1.y) * sin_phi,
            (self.p2.x - self.p1.x) * sin_phi + (self.p2.y - self.p1.y) * cos_phi,
            self.p2.z - self.p1.z,
        );

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
