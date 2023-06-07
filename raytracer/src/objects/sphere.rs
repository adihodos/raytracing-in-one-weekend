use math::vec3::{self, length_squared};

use crate::aabb3::Aabb;
use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::types::{Point, Ray, Real};

#[derive(Clone)]
pub struct Sphere {
    pub center: Point,
    pub radius: Real,
    pub mtl: std::sync::Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point, radius: Real, mtl: std::sync::Arc<dyn Material>) -> Sphere {
        Sphere {
            center,
            radius,
            mtl,
        }
    }

    fn get_uv(p: Point) -> (f32, f32) {
        //
        // p: a given point on the sphere of radius one, centered at the origin.
        // u: returned value [0,1] of angle around the Y axis from X=-1.
        // v: returned value [0,1] of angle from Y=-1 to Y=+1.
        //     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
        //     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
        //     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>

        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + std::f64::consts::PI as Real;

        (
            phi / (2f64 * std::f64::consts::PI) as Real,
            theta / std::f64::consts::PI as Real,
        )
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = length_squared(r.direction);
        let half_b = vec3::dot(oc, r.direction);

        let c = length_squared(oc) - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0f32 {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        //
        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let t = root;
        let p = r.at(t);
        let outward_normal = (p - self.center) / self.radius;
        let (u, v) = Self::get_uv(outward_normal);

        Some(HitRecord::new(
            p,
            outward_normal,
            r,
            t,
            self.mtl.clone(),
            u,
            v,
        ))
    }

    fn bounding_box(&self, _time0: Real, _time1: Real) -> Option<Aabb> {
        Some(Aabb::new(
            self.center - Point::broadcast(self.radius),
            self.center + Point::broadcast(self.radius),
        ))
    }
}

#[derive(Clone)]
pub struct MovingSphere {
    pub center0: Point,
    pub center1: Point,
    pub radius: Real,
    pub time0: Real,
    pub time1: Real,
    pub mtl: std::sync::Arc<dyn Material>,
}

impl MovingSphere {
    pub fn new(
        center0: Point,
        center1: Point,
        time0: Real,
        time1: Real,
        radius: Real,
        mtl: std::sync::Arc<dyn Material>,
    ) -> MovingSphere {
        MovingSphere {
            center0,
            center1,
            radius,
            mtl,
            time0,
            time1,
        }
    }

    fn center(&self, time: Real) -> Point {
        self.center0
            + ((time - self.time0) / (self.time1 - self.time0)) * (self.center1 - self.center0)
    }
}

impl Hittable for MovingSphere {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        let oc = r.origin - self.center(r.time);
        let a = length_squared(r.direction);
        let half_b = vec3::dot(oc, r.direction);

        let c = length_squared(oc) - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0f32 {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        //
        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let t = root;
        let p = r.at(t);
        let outward_normal = (p - self.center(r.time)) / self.radius;
        let (u, v) = Sphere::get_uv(outward_normal);

        Some(HitRecord::new(
            p,
            outward_normal,
            r,
            t,
            self.mtl.clone(),
            u,
            v,
        ))
    }

    fn bounding_box(&self, time0: Real, time1: Real) -> Option<crate::aabb3::Aabb> {
        let box_t0 = Aabb::new(
            self.center(time0) - Point::broadcast(self.radius),
            self.center(time0) + Point::broadcast(self.radius),
        );

        let box_t1 = Aabb::new(
            self.center(time1) - Point::broadcast(self.radius),
            self.center(time1) + Point::broadcast(self.radius),
        );

        Some(crate::aabb3::merge_aabbs(&box_t0, &box_t1))
    }
}
