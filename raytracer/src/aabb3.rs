use crate::types::{Ray, Real, Vec3};

#[derive(Copy, Clone, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new<P: std::convert::Into<Vec3>>(min: P, max: P) -> Aabb {
        Aabb {
            min: min.into(),
            max: max.into(),
        }
    }

    pub fn hit(&self, r: &Ray, tmin: f32, tmax: f32) -> bool {
        let mut tmin = tmin;
        let mut tmax = tmax;

        for a in 0..3 {
            let inv_d = 1 as Real / r.direction[a as usize];

            let t0 = crate::types::ffmin(
                (self.min[a as usize] - r.origin[a as usize]) * inv_d,
                (self.max[a as usize] - r.origin[a as usize]) * inv_d,
            );

            let t1 = crate::types::ffmax(
                (self.min[a as usize] - r.origin[a as usize]) * inv_d,
                (self.max[a as usize] - r.origin[a as usize]) * inv_d,
            );

            tmin = crate::types::ffmax(t0, tmin);
            tmax = crate::types::ffmin(t1, tmax);

            if tmax <= tmin {
                return false;
            }
        }

        true
    }
}
pub fn merge_aabbs(a: &Aabb, b: &Aabb) -> Aabb {
    let min = Vec3::new(
        a.min.x.min(b.min.x),
        a.min.y.min(b.min.y),
        a.min.z.min(b.min.z),
    );

    let max = Vec3::new(
        a.max.x.max(b.max.x),
        a.max.y.max(b.max.y),
        a.max.z.max(b.max.z),
    );

    Aabb::new(min, max)
}

impl Default for Aabb {
    fn default() -> Aabb {
        Aabb::new(
            Vec3::broadcast(std::f32::MAX),
            Vec3::broadcast(std::f32::MIN),
        )
    }
}
