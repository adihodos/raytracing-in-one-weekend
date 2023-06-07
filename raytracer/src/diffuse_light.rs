use std::sync::Arc;

use crate::{
    material::Material, solid_color_texture::SolidColorTexture, texture::Texture, types::Color,
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
        u: crate::types::Real,
        v: crate::types::Real,
        point: crate::types::Point,
    ) -> Color {
        self.emit.value(u, v, point)
    }
}
