use std::sync::Arc;

use math::vec3::{dot, normalize};

use crate::hittable::HitRecord;
use crate::material::{Material, ScatterRecord};
use crate::onb::Onb;
use crate::pdf::CosinePdf;
use crate::solid_color_texture::SolidColorTexture;
use crate::texture::Texture;
use crate::types::{random_cosine_direction, Color, Ray, Real};

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
    fn scatter(&self, _ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        let albedo = self.albedo.value(hit_record.u, hit_record.v, hit_record.p);

        Some(ScatterRecord::PdfRec {
            pdf: Arc::new(CosinePdf {
                uvw: hit_record.normal.into(),
            }),
            attenuation: albedo,
        })
    }

    fn scattering_pdf(&self, _ray: &Ray, hit_record: &HitRecord, scattered: &Ray) -> Real {
        let cosine = dot(hit_record.normal, normalize(scattered.direction));
        if cosine < 0f32 {
            0f32
        } else {
            cosine / std::f32::consts::PI
        }
    }
}
