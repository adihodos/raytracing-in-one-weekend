use crate::hittable::HitRecord;
use crate::material::{Material, ScatterRecord};
use crate::types::{Color, Ray};

#[derive(Copy, Clone, Debug)]
pub struct Metal {
    pub albedo: Color,
}

impl Metal {
    pub fn new(albedo: Color) -> Metal {
        Metal { albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        use math::vec3::{are_on_the_same_plane_side, normalize, reflect_unit_vector};
        let reflected = reflect_unit_vector(normalize(ray.direction), hit_record.normal);

        if are_on_the_same_plane_side(reflected, hit_record.normal) {
            Some(ScatterRecord {
                ray: Ray::new(hit_record.p, reflected),
                attenuation: self.albedo,
            })
        } else {
            None
        }
    }
}
