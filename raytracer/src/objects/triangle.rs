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

impl std::ops::Index<usize> for Triangle {
    type Output = Point;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.v0,
            1 => &self.v1,
            2 => &self.v2,
            _ => panic!("Index must be in the [0, 2] range"),
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
        let containment_tests_failed = [(0, 1), (1, 2), (2, 0)].iter().any(|vertex_indices| {
            // direction vector along the edge
            let edge_vec = self[vertex_indices.1] - self[vertex_indices.0];
            // direction vector from the vertex to the intersection point with the ray
            let intersect_point_vec = p - self[vertex_indices.0];
            // orthogonal vector to the above two vectors
            let orthogonal_vec = cross(edge_vec, intersect_point_vec);

            !are_on_the_same_plane_side(orthogonal_vec, self.normal)
        });

        if containment_tests_failed {
            //
            // point is on the plane defined by the triangle's vertices but
            // outside the triangle
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
