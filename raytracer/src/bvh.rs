use std::{cmp::Ordering, sync::Arc};

use crate::{
    aabb3::Aabb,
    hittable::Hittable,
    types::{random_int, Real},
};

pub struct BvhNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox: Aabb,
}

impl BvhNode {
    pub fn new(l: &mut [Arc<dyn Hittable>], time0: Real, time1: Real) -> Arc<dyn Hittable> {
        let cmp_axis = random_int(0, 2);
        let cmp_fn = match cmp_axis {
            0 => box_x_compare,
            1 => box_y_compare,
            2 => box_z_compare,
            _ => panic!("But how ????"),
        };

        let (left, right) = if l.len() == 1 {
            (l[0].clone(), l[0].clone())
        } else if l.len() == 2 {
            if cmp_fn(&l[0], &l[1]) == Ordering::Less {
                (l[0].clone(), l[1].clone())
            } else {
                (l[1].clone(), l[0].clone())
            }
        } else {
            l.sort_by(cmp_fn);
            let mid = l.len() / 2;

            let left = Self::new(&mut l[..mid], time0, time1);
            let right = Self::new(&mut l[mid..], time0, time1);

            (left as Arc<dyn Hittable>, right as Arc<dyn Hittable>)
        };

        let bbox_left = left.bounding_box(time0, time1);
        let bbox_right = right.bounding_box(time0, time1);

        if bbox_left.is_none() || bbox_right.is_none() {
            panic!("No bounding box in bvh_node constructor");
        }

        Arc::new(Self {
            left,
            right,
            bbox: crate::aabb3::merge_aabbs(&bbox_left.unwrap(), &bbox_right.unwrap()),
        })
    }
}

impl Hittable for BvhNode {
    fn hit(
        &self,
        r: &crate::types::Ray,
        t_min: crate::types::Real,
        t_max: crate::types::Real,
    ) -> Option<crate::hittable::HitRecord> {
        if !self.bbox.hit(r, t_min, t_max) {
            return None;
        }

        let hit_left = self.left.hit(r, t_min, t_max);
        let hit_right = self.right.hit(
            r,
            t_min,
            if let Some(hl) = hit_left.as_ref() {
                hl.t
            } else {
                t_max
            },
        );

        hit_left
            .into_iter()
            .chain(hit_right.into_iter())
            .reduce(|a, b| if a.t < b.t { a } else { b })
    }

    fn bounding_box(&self, _time0: crate::types::Real, _time1: crate::types::Real) -> Option<Aabb> {
        Some(self.bbox)
    }
}

fn box_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>, axis: usize) -> Ordering {
    let box_a = a
        .bounding_box(0f32, 0f32)
        .expect("No bounding box in BVH node constructor");
    let box_b = b
        .bounding_box(0f32, 0f32)
        .expect("No bounding box in BVH node constructor");

    box_a.min[axis].partial_cmp(&box_b.min[axis]).unwrap()
}

fn box_x_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> Ordering {
    box_compare(a, b, 0)
}

fn box_y_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> Ordering {
    box_compare(a, b, 1)
}

fn box_z_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> Ordering {
    box_compare(a, b, 2)
}
