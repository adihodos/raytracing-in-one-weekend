use crate::hittable::HitRecord;
use crate::material::{Material, ScatterRecord};
use crate::types::{Color, Ray, Vec3};

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
        use math::vec3::{are_on_the_same_plane_side, reflect};
        let reflected = hit_record.normal + reflect(ray.direction, hit_record.normal);

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
