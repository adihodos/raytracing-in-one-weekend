use crate::material::Material;
use crate::types::{Point, Ray, Real, Vec3};

#[derive(Clone)]
pub struct HitRecord {
    pub p: Point,
    pub normal: Vec3,
    pub t: Real,
    pub mtl: std::rc::Rc<dyn Material>,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(
        p: Point,
        outward_normal: Vec3,
        ray: &Ray,
        t: Real,
        mtl: std::rc::Rc<dyn Material>,
    ) -> HitRecord {
        let front_face = !math::vec3::are_on_the_same_plane_side(ray.direction, outward_normal);
        HitRecord {
            p,
            normal: if front_face {
                outward_normal
            } else {
                -outward_normal
            },
            t,
            mtl: std::rc::Rc::clone(&mtl),
            front_face,
        }
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord>;
}
