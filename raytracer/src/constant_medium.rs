use std::sync::Arc;

use math::vec3::length;

use crate::{
    hittable::{HitRecord, Hittable},
    isotropic::Isotropic,
    material::Material,
    texture::Texture,
    types::{random_real, Color, Real},
};

pub struct ConstantMedium {
    boundary: Arc<dyn Hittable>,
    phase_function: Arc<dyn Material>,
    neg_inv_density: Real,
}

impl ConstantMedium {
    pub fn new(boundary: Arc<dyn Hittable>, mat: Arc<dyn Texture>, density: Real) -> Self {
        Self {
            boundary,
            phase_function: Arc::new(Isotropic { albedo: mat }),
            neg_inv_density: -1 as Real / density,
        }
    }

    pub fn from_colored_object<T: Into<Color>>(
        boundary: Arc<dyn Hittable>,
        color: T,
        density: Real,
    ) -> Self {
        Self {
            boundary,
            phase_function: Arc::new(Isotropic::from(color)),
            neg_inv_density: -1 as Real / density,
        }
    }
}

impl Hittable for ConstantMedium {
    fn bounding_box(&self, time0: Real, time1: Real) -> Option<crate::aabb3::Aabb> {
        self.boundary.bounding_box(time0, time1)
    }

    fn hit(
        &self,
        r: &crate::types::Ray,
        t_min: Real,
        t_max: Real,
    ) -> Option<crate::hittable::HitRecord> {
        let mut rec1 = self
            .boundary
            .hit(r, std::f32::MIN as Real, std::f32::MAX as Real)?;

        let mut rec2 = self
            .boundary
            .hit(r, rec1.t + 0.0001_f32 as Real, std::f32::MAX as Real)?;

        if rec1.t < t_min {
            rec1.t = t_min;
        }

        if rec2.t > t_max {
            rec2.t = t_max;
        }

        if rec1.t >= rec2.t {
            return None;
        }

        if rec1.t < 0 as Real {
            rec1.t = 0 as Real;
        }

        let ray_length = length(r.direction);
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = self.neg_inv_density * (random_real().ln()) as Real;

        if hit_distance > distance_inside_boundary {
            return None;
        }

        let t = rec1.t + hit_distance / ray_length;
        Some(HitRecord {
            p: r.at(t),
            normal: (1f32, 0f32, 0f32).into(),
            t,
            mtl: self.phase_function.clone(),
            front_face: true,
            u: rec1.u,
            v: rec1.v,
        })
    }
}
