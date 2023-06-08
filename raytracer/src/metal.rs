use crate::hittable::HitRecord;
use crate::material::{Material, ScatterRecord};
use crate::types::{Color, Ray, Real};

#[derive(Copy, Clone, Debug)]
pub struct Metal {
    pub albedo: Color,
    pub fuzziness: Real,
}

impl Metal {
    pub fn new<T: Into<Color>>(albedo: T, fuzziness: Real) -> Metal {
        Metal {
            albedo: albedo.into(),
            fuzziness,
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        use crate::types::random_in_unit_sphere;
        use math::vec3::{are_on_the_same_plane_side, normalize, reflect_unit_vector};

        let reflected = reflect_unit_vector(ray.direction, normalize(hit_record.normal));
        let scattered = reflected + self.fuzziness * random_in_unit_sphere();

        if are_on_the_same_plane_side(scattered, hit_record.normal) {
            Some(ScatterRecord {
                ray: Ray::new(hit_record.p, scattered, ray.time),
                albedo: self.albedo,
                pdf: 0 as Real,
            })
        } else {
            None
        }
    }
}
