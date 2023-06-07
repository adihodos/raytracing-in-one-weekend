use std::sync::Arc;

use crate::{
    aabb3::Aabb,
    hittable::{HitRecord, Hittable},
    types::{degrees_to_radians, Point, Ray, Real, Vec3},
};

pub struct Translate {
    pub obj: Arc<dyn Hittable>,
    pub offset: Vec3,
}

impl Hittable for Translate {
    fn bounding_box(
        &self,
        time0: crate::types::Real,
        time1: crate::types::Real,
    ) -> Option<crate::aabb3::Aabb> {
        self.obj
            .bounding_box(time0, time1)
            .map(|bbox| crate::aabb3::Aabb {
                min: bbox.min + self.offset,
                max: bbox.max + self.offset,
            })
    }

    fn hit(
        &self,
        r: &crate::types::Ray,
        t_min: crate::types::Real,
        t_max: crate::types::Real,
    ) -> Option<crate::hittable::HitRecord> {
        let translated_ray = Ray::new(r.origin - self.offset, r.direction, r.time);
        self.obj.hit(&translated_ray, t_min, t_max).map(|hit_data| {
            HitRecord::new(
                hit_data.p + self.offset,
                hit_data.normal,
                &translated_ray,
                hit_data.t,
                hit_data.mtl,
                hit_data.u,
                hit_data.v,
            )
        })
    }
}

pub struct RotateY {
    obj: Arc<dyn Hittable>,
    sin_theta: Real,
    cos_theta: Real,
    bbox: Aabb,
}

impl RotateY {
    pub fn new(obj: Arc<dyn Hittable>, angle: Real) -> Self {
        let rads = degrees_to_radians(angle);
        let (sin_theta, cos_theta) = rads.sin_cos();

        let bbox = obj
            .bounding_box(0 as Real, 1 as Real)
            .expect("Object does not have a bounding box ...");

        let mut min = Point::broadcast(std::f32::MAX as Real);
        let mut max = Point::broadcast(std::f32::MIN as Real);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as Real * bbox.max.x + (1 as Real - i as Real) * bbox.min.x;
                    let y = j as Real * bbox.max.y + (1 as Real - j as Real) * bbox.min.y;
                    let z = k as Real * bbox.max.z + (1 as Real - k as Real) * bbox.min.z;

                    let tester = Vec3::new(
                        cos_theta * x + sin_theta * z,
                        y,
                        -sin_theta * x + cos_theta * z,
                    );

                    for c in 0..3 {
                        if tester[c as usize] > max[c as usize] {
                            max[c as usize] = tester[c as usize];
                        }

                        if tester[c as usize] < min[c as usize] {
                            min[c as usize] = tester[c as usize];
                        }
                    }
                }
            }
        }

        Self {
            obj,
            sin_theta,
            cos_theta,
            bbox: Aabb { min, max },
        }
    }
}

impl Hittable for RotateY {
    fn bounding_box(&self, _time0: Real, _time1: Real) -> Option<Aabb> {
        Some(self.bbox)
    }

    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        let mut origin = r.origin;
        let mut direction = r.direction;

        origin[0] = self.cos_theta * r.origin[0] - self.sin_theta * r.origin[2];
        origin[2] = self.sin_theta * r.origin[0] + self.cos_theta * r.origin[2];

        direction[0] = self.cos_theta * r.direction[0] - self.sin_theta * r.direction[2];
        direction[2] = self.sin_theta * r.direction[0] + self.cos_theta * r.direction[2];

        let rotated_r = Ray::new(origin, direction, r.time);

        self.obj.hit(&rotated_r, t_min, t_max).map(|hitrec| {
            let mut p = hitrec.p;
            let mut n = hitrec.normal;

            p[0] = self.cos_theta * hitrec.p[0] + self.sin_theta * hitrec.p[2];
            p[2] = -self.sin_theta * hitrec.p[0] + self.cos_theta * hitrec.p[2];

            n[0] = self.cos_theta * hitrec.normal[0] + self.sin_theta * hitrec.normal[2];
            n[2] = -self.sin_theta * hitrec.normal[0] + self.cos_theta * hitrec.normal[2];

            HitRecord {
                p,
                normal: n,
                ..hitrec
            }
        })
    }
}
