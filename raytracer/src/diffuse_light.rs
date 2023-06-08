use std::sync::Arc;

use math::ray;

use crate::{
    hittable::HitRecord,
    material::Material,
    solid_color_texture::SolidColorTexture,
    texture::Texture,
    types::{Color, Ray, Real},
};

pub struct DiffuseLight {
    emit: Arc<dyn Texture>,
}

impl DiffuseLight {
    pub fn with_texture(emit: Arc<dyn Texture>) -> Self {
        Self { emit }
    }
}

impl<T> std::convert::From<T> for DiffuseLight
where
    T: Into<Color>,
{
    fn from(color: T) -> Self {
        Self {
            emit: Arc::new(SolidColorTexture::new(color)),
        }
    }
}

impl Material for DiffuseLight {
    fn scatter(
        &self,
        _ray: &crate::types::Ray,
        _hit_record: &crate::hittable::HitRecord,
    ) -> Option<crate::material::ScatterRecord> {
        None
    }

    fn emitted(
        &self,
        ray: &Ray,
        hit_rec: &HitRecord,
        u: crate::types::Real,
        v: crate::types::Real,
        point: crate::types::Point,
    ) -> Color {
        if hit_rec.front_face {
            self.emit.value(u, v, point)
        } else {
            Color::broadcast(0 as Real)
        }
    }
}
