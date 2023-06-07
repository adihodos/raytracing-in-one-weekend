use crate::hittable::HitRecord;
use crate::material::{Material, ScatterRecord};
use crate::solid_color_texture::SolidColorTexture;
use crate::texture::Texture;
use crate::types::{random_unit_vector, Color, Ray};

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

        Some(ScatterRecord {
            ray: Ray::new(hit_record.p, scatter_direction, ray.time),
            attenuation: self.albedo.value(hit_record.u, hit_record.v, hit_record.p),
        })
    }
}
