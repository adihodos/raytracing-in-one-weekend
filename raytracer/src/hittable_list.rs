#![allow(dead_code)]

use crate::hittable::{HitRecord, Hittable};
use crate::types::{Ray, Real};

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
}
