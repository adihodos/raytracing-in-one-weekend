use std::sync::Arc;

use crate::hittable::{HitRecord, Hittable};

pub struct FlipFace {
    pub obj: Arc<dyn Hittable>,
}

impl Hittable for FlipFace {
    fn bounding_box(
        &self,
        time0: crate::types::Real,
        time1: crate::types::Real,
    ) -> Option<crate::aabb3::Aabb> {
        self.obj.bounding_box(time0, time1)
    }

    fn hit(
        &self,
        r: &crate::types::Ray,
        t_min: crate::types::Real,
        t_max: crate::types::Real,
    ) -> Option<crate::hittable::HitRecord> {
        self.obj.hit(r, t_min, t_max).map(|hit_rec| HitRecord {
            front_face: !hit_rec.front_face,
            ..hit_rec
        })
    }
}
