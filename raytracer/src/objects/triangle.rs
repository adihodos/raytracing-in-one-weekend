use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::types::{Point, Ray, Real, Vec3};

#[derive(Clone)]
pub struct Triangle {
    pub v0: Point,
    pub v1: Point,
    pub v2: Point,
    pub normal: Vec3,
    pub mtl: std::sync::Arc<dyn Material>,
}

impl Triangle {
    pub fn new(v0: Point, v1: Point, v2: Point, mtl: std::sync::Arc<dyn Material>) -> Triangle {
        Triangle {
            v0,
            v1,
            v2,
            normal: math::vec3::normalize(math::vec3::cross(v1 - v0, v2 - v0)),
            mtl,
        }
    }
}

impl Hittable for Triangle {
    fn hit(&self, ray: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        use math::vec3::{are_on_the_same_plane_side, cross, dot};

        //
        // check if the ray hits the triangle plane (use v0 as origin)
        let d = dot(self.normal, self.v0);

        const EPSILON: Real = 1.0E-5 as Real;
        let b_dot_n = dot(ray.direction, self.normal);

        if b_dot_n.abs() < EPSILON {
            //
            // ray is parallel or contained in the triangle's plane
            return None;
        }

        //
        // compute point of intersection on the triangle's plane
        let a_dot_n = dot(ray.origin, self.normal);
        let t = (d - a_dot_n) / b_dot_n;

        if !(t < t_max && t > t_min) {
            //
            // intersection point is behind the ray
            return None;
        }

        let p = ray.at(t);

        //
        // check if the point lies inside the triangle

        //
        // Check edge from v0 to v1

        // vector along the edge of the triangle
        let edge_vector = self.v1 - self.v0;
        // vector from this vertex to the intersection point
        let vertex_to_point_vector = p - self.v0;
        // vector perpendicular to the edge and intersection point vector
        let perp_vector = cross(edge_vector, vertex_to_point_vector);

        if !are_on_the_same_plane_side(perp_vector, self.normal) {
            return None;
        }

        //
        // Check edge from v1 to v2
        let edge_vector = self.v2 - self.v1;
        let vertex_to_point_vector = p - self.v1;
        let perp_vector = cross(edge_vector, vertex_to_point_vector);

        if !are_on_the_same_plane_side(perp_vector, self.normal) {
            return None;
        }

        //
        // Check edge from v2 to v0
        let edge_vector = self.v0 - self.v2;
        let vertex_to_point_vector = p - self.v2;
        let perp_vector = cross(edge_vector, vertex_to_point_vector);

        if !are_on_the_same_plane_side(perp_vector, self.normal) {
            return None;
        }

        //
        // Point lies inside the triangle
        Some(HitRecord::new(
            p,
            self.normal,
            ray,
            t,
            std::sync::Arc::clone(&self.mtl),
        ))
    }
}
