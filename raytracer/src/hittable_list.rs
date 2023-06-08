#![allow(dead_code)]

use crate::hittable::{HitRecord, Hittable};
use crate::types::{random_int, Ray, Real};

#[derive(Clone)]
pub struct HittableList {
    objects: Vec<std::sync::Arc<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> HittableList {
        HittableList {
            objects: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear()
    }

    pub fn add(&mut self, object: std::sync::Arc<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl std::iter::FromIterator<std::sync::Arc<dyn Hittable>> for HittableList {
    fn from_iter<T>(i: T) -> Self
    where
        T: std::iter::IntoIterator<Item = std::sync::Arc<dyn Hittable>>,
    {
        HittableList {
            objects: Vec::from_iter(i),
        }
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        self.objects
            .iter()
            .filter_map(|obj| obj.hit(r, t_min, t_max))
            .min_by(|hit_rec_a, hit_rec_b| {
                if hit_rec_a.t < hit_rec_b.t {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            })
    }

    fn bounding_box(&self, time0: Real, time1: Real) -> Option<crate::aabb3::Aabb> {
        self.objects
            .iter()
            .filter_map(|object| object.bounding_box(time0, time1))
            .reduce(|accum_box, this_box| crate::aabb3::merge_aabbs(&accum_box, &this_box))
    }

    fn pdf_value(&self, o: crate::types::Point, v: crate::types::Vec3) -> Real {
        if self.objects.is_empty() {
            return 0 as Real;
        }

        let weight = 1 as Real / self.objects.len() as Real;

        self.objects
            .iter()
            .fold(0 as Real, |sum, obj| sum + weight * obj.pdf_value(o, v))
    }

    fn random(&self, v: crate::types::Vec3) -> crate::types::Vec3 {
        let num_objects = (self.objects.len() - 1) as i32;

        self.objects[random_int(0, num_objects) as usize].random(v)
    }
}
