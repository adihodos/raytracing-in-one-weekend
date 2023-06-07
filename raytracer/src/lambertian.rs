use crate::hittable::HitRecord;
use crate::material::{Material, ScatterRecord};
use crate::types::{random_unit_vector, Color, Ray};

#[derive(Copy, Clone, Debug)]
pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        let scatter_direction = hit_record.normal + random_unit_vector();
        Some(ScatterRecord {
            ray: Ray::new(hit_record.p, scatter_direction, ray.time),
            attenuation: self.albedo,
        })
    }
}
