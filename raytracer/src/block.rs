use std::sync::Arc;

use crate::{
    hittable::Hittable,
    hittable_list::HittableList,
    material::Material,
    rectangles::{XYRect, XZRect, YZRect},
    types::{Color, Point},
};

pub struct Block {
    box_min: Point,
    box_max: Point,
    sides: HittableList,
}

impl Block {
    pub fn new<P: Into<Point>>(p0: P, p1: P, mtl: Arc<dyn Material>) -> Self {
        let mut sides = HittableList::new();
        let p0: Point = p0.into();
        let p1: Point = p1.into();

        sides.add(Arc::new(XYRect {
            x0: p0.x,
            x1: p1.x,
            y0: p0.y,
            y1: p1.y,
            k: p1.z,
            mtl: mtl.clone(),
        }));

        sides.add(Arc::new(XYRect {
            x0: p0.x,
            x1: p1.x,
            y0: p0.y,
            y1: p1.y,
            k: p0.z,
            mtl: mtl.clone(),
        }));

        sides.add(Arc::new(XZRect {
            x0: p0.x,
            x1: p1.x,
            z0: p0.z,
            z1: p1.z,
            k: p1.y,
            mtl: mtl.clone(),
        }));

        sides.add(Arc::new(XZRect {
            x0: p0.x,
            x1: p1.x,
            z0: p0.z,
            z1: p1.z,
            k: p0.y,
            mtl: mtl.clone(),
        }));

        sides.add(Arc::new(YZRect {
            y0: p0.y,
            y1: p1.y,
            z0: p0.z,
            z1: p1.z,
            k: p1.x,
            mtl: mtl.clone(),
        }));

        sides.add(Arc::new(YZRect {
            y0: p0.y,
            y1: p1.y,
            z0: p0.z,
            z1: p1.z,
            k: p0.x,
            mtl: mtl.clone(),
        }));

        Self {
            box_min: p0,
            box_max: p1,
            sides,
        }
    }
}

impl Hittable for Block {
    fn bounding_box(
        &self,
        _time0: crate::types::Real,
        _time1: crate::types::Real,
    ) -> Option<crate::aabb3::Aabb> {
        Some(crate::aabb3::Aabb {
            min: self.box_min,
            max: self.box_max,
        })
    }

    fn hit(
        &self,
        r: &crate::types::Ray,
        t_min: crate::types::Real,
        t_max: crate::types::Real,
    ) -> Option<crate::hittable::HitRecord> {
        self.sides.hit(r, t_min, t_max)
    }
}
