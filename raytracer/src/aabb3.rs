use math::vec3::TVec3;

use crate::types::{Mat4, Ray, Real, Vec3};

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

    pub fn add_point(&mut self, p: TVec3<Real>) {
        self.min = math::vec3::min(self.min, p);
        self.max = math::vec3::max_sv(self.max, p);
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

pub fn transform(mat: &Mat4, aabb: &Aabb) -> Aabb {
    let xmin = mat[0] * aabb.min.x;
    let xmax = mat[0] * aabb.max.x;

    let ymin = mat[1] * aabb.min.y;
    let ymax = mat[1] * aabb.max.y;

    let zmin = mat[2] * aabb.min.z;
    let zmax = mat[2] * aabb.max.z;

    use math::vec4::{max, min};

    Aabb {
        min: (min(xmin, xmax) + min(ymin, ymax) + min(zmin, zmax) + mat.column(3)).xyz(),
        max: (max(xmin, xmax) + max(ymin, ymax) + max(zmin, zmax) + mat.column(3)).xyz(),
    }
}
