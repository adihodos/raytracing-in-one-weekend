use std::sync::Arc;

use crate::{
    material::{Material, ScatterRecord},
    solid_color_texture::SolidColorTexture,
    texture::Texture,
    types::{random_in_unit_sphere, Color, Ray, Real},
};

pub struct Isotropic {
    pub albedo: Arc<dyn Texture>,
}

impl<T> std::convert::From<T> for Isotropic
where
    T: Into<Color>,
{
    fn from(color: T) -> Self {
        Self {
            albedo: Arc::new(SolidColorTexture::new(color)),
        }
    }
}

impl Material for Isotropic {
    fn scatter(
        &self,
        _ray: &crate::types::Ray,
        hit_record: &crate::hittable::HitRecord,
    ) -> Option<crate::material::ScatterRecord> {
        Some(ScatterRecord::SpecularRec {
            ray: Ray::new(hit_record.p, random_in_unit_sphere(), hit_record.t),
            attenuation: self.albedo.value(hit_record.u, hit_record.v, hit_record.p),
        })
    }
}
