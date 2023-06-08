use math::vec3::{dot, normalize};

use crate::hittable::HitRecord;
use crate::material::{Material, ScatterRecord};
use crate::solid_color_texture::SolidColorTexture;
use crate::texture::Texture;
use crate::types::{random_unit_vector, Color, Ray, Real};

#[derive(Clone)]
pub struct Lambertian {
    pub albedo: std::sync::Arc<dyn Texture>,
}

impl Lambertian {
    pub fn new<T>(albedo: T) -> Lambertian
    where
        T: Into<Color>,
    {
        Lambertian {
            albedo: std::sync::Arc::new(SolidColorTexture::new(albedo)),
        }
    }

    pub fn from_texture(albedo: std::sync::Arc<dyn Texture>) -> Lambertian {
        Lambertian { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        let mut scatter_direction = hit_record.normal + random_unit_vector();

        if math::vec3::is_near_zero(scatter_direction) {
            scatter_direction = hit_record.normal;
        }

        let scattered_ray = Ray::new(hit_record.p, normalize(scatter_direction), ray.time);
        let albedo = self.albedo.value(hit_record.u, hit_record.v, hit_record.p);
        let pdf = dot(hit_record.normal, scatter_direction) / std::f32::consts::PI as Real;

        Some(ScatterRecord {
            ray: scattered_ray,
            albedo,
            pdf,
        })
    }

    fn scattering_pdf(&self, ray: &Ray, hit_record: &HitRecord, scattered: &ScatterRecord) -> Real {
        let cosine = dot(hit_record.normal, normalize(scattered.ray.direction));
        if cosine < 0f32 {
            0f32
        } else {
            cosine / std::f32::consts::PI
        }
    }
}
