use std::sync::Arc;

use math::vec3::{dot, length, length_squared};

use crate::{
    aabb3::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    types::{random_real_range, Ray, Real, Vec3},
};

pub struct XYRect {
    pub x0: Real,
    pub x1: Real,
    pub y0: Real,
    pub y1: Real,
    pub k: Real,
    pub mtl: Arc<dyn Material>,
}

impl Hittable for XYRect {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        let t = (self.k - r.origin.z) / r.direction.z;

        if t < t_min || t > t_max {
            return None;
        }

        let x = r.origin.x + t * r.direction.x;
        let y = r.origin.y + t * r.direction.y;

        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        Some(HitRecord::new(
            r.at(t),
            Vec3::new(0 as Real, 0 as Real, 1 as Real),
            r,
            t,
            self.mtl.clone(),
            (x - self.x0) / (self.x1 - self.x0),
            (y - self.y0) / (self.y1 - self.y0),
        ))
    }

    fn bounding_box(&self, _t0: Real, _t11: Real) -> Option<Aabb> {
        Some(Aabb::new(
            Vec3::new(self.x0, self.y0, self.k - 0.0001 as Real),
            Vec3::new(self.x1, self.y1, self.k + 0.0001 as Real),
        ))
    }

    fn pdf_value(&self, origin: crate::types::Point, v: Vec3) -> Real {
        self.hit(
            &Ray::new(origin, v, 0 as Real),
            0.001 as Real,
            std::f32::MAX as Real,
        )
        .map_or(0 as Real, |hit_rec| {
            let area = (self.x1 - self.x0) * (self.y1 - self.y0);
            let distance_squared = hit_rec.t * hit_rec.t * length_squared(v);
            let cosine = (dot(v, hit_rec.normal) / length(v)).abs();

            distance_squared / (cosine * area)
        })
    }

    fn random(&self, origin: Vec3) -> Vec3 {
        let random_point = Vec3 {
            x: random_real_range(self.x0, self.x1),
            y: random_real_range(self.y0, self.y1),
            z: self.k,
        };

        random_point - origin
    }
}

//
//

pub struct XZRect {
    pub x0: Real,
    pub x1: Real,
    pub z0: Real,
    pub z1: Real,
    pub k: Real,
    pub mtl: Arc<dyn Material>,
}

impl Hittable for XZRect {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        let t = (self.k - r.origin.y) / r.direction.y;

        if t < t_min || t > t_max {
            return None;
        }

        let x = r.origin.x + t * r.direction.x;
        let z = r.origin.z + t * r.direction.z;

        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }

        Some(HitRecord::new(
            r.at(t),
            Vec3::new(0 as Real, 1 as Real, 0 as Real),
            r,
            t,
            self.mtl.clone(),
            (x - self.x0) / (self.x1 - self.x0),
            (z - self.z0) / (self.z1 - self.z0),
        ))
    }

    fn bounding_box(&self, _t0: Real, _t11: Real) -> Option<Aabb> {
        Some(Aabb::new(
            Vec3::new(self.x0, self.k - 0.0001 as Real, self.z0),
            Vec3::new(self.x1, self.k + 0.0001 as Real, self.z1),
        ))
    }

    fn pdf_value(&self, origin: crate::types::Point, v: Vec3) -> Real {
        self.hit(
            &Ray::new(origin, v, 0 as Real),
            0.001 as Real,
            std::f32::MAX as Real,
        )
        .map_or(0 as Real, |hit_rec| {
            let area = (self.x1 - self.x0) * (self.z1 - self.z0);
            let distance_squared = hit_rec.t * hit_rec.t * length_squared(v);
            let cosine = (dot(v, hit_rec.normal) / length(v)).abs();

            distance_squared / (cosine * area)
        })
    }

    fn random(&self, origin: Vec3) -> Vec3 {
        let random_point = Vec3 {
            x: random_real_range(self.x0, self.x1),
            y: self.k,
            z: random_real_range(self.z0, self.z1),
        };

        random_point - origin
    }
}

//
//
pub struct YZRect {
    pub y0: Real,
    pub y1: Real,
    pub z0: Real,
    pub z1: Real,
    pub k: Real,
    pub mtl: Arc<dyn Material>,
}

impl Hittable for YZRect {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        let t = (self.k - r.origin.x) / r.direction.x;

        if t < t_min || t > t_max {
            return None;
        }

        let y = r.origin.y + t * r.direction.y;
        let z = r.origin.z + t * r.direction.z;

        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }

        Some(HitRecord::new(
            r.at(t),
            Vec3::new(1 as Real, 0 as Real, 0 as Real),
            r,
            t,
            self.mtl.clone(),
            (y - self.y0) / (self.y1 - self.y0),
            (z - self.z0) / (self.z1 - self.z0),
        ))
    }

    fn bounding_box(&self, _t0: Real, _t11: Real) -> Option<Aabb> {
        Some(Aabb::new(
            Vec3::new(self.k - 0.0001 as Real, self.y0, self.z0),
            Vec3::new(self.k + 0.0001 as Real, self.y1, self.z1),
        ))
    }

    fn pdf_value(&self, origin: crate::types::Point, v: Vec3) -> Real {
        self.hit(
            &Ray::new(origin, v, 0 as Real),
            0.001 as Real,
            std::f32::MAX as Real,
        )
        .map_or(0 as Real, |hit_rec| {
            let area = (self.y1 - self.y0) * (self.z1 - self.z0);
            let distance_squared = hit_rec.t * hit_rec.t * length_squared(v);
            let cosine = (dot(v, hit_rec.normal) / length(v)).abs();

            distance_squared / (cosine * area)
        })
    }

    fn random(&self, origin: Vec3) -> Vec3 {
        let random_point = Vec3 {
            x: self.k,
            y: random_real_range(self.y0, self.y1),
            z: random_real_range(self.z0, self.z1),
        };

        random_point - origin
    }
}
