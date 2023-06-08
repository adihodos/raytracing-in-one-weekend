use std::sync::Arc;

use crate::hittable::HitRecord;
use crate::pdf::Pdf;
use crate::types::{Color, Point, Ray, Real};

pub enum ScatterRecord {
    SpecularRec {
        ray: Ray,
        attenuation: Color,
    },
    PdfRec {
        pdf: Arc<dyn Pdf>,
        attenuation: Color,
    },
}

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord>;

    fn scattering_pdf(&self, _ray: &Ray, _hit_record: &HitRecord, _scattered: &Ray) -> Real {
        0 as Real
    }

    fn emitted(
        &self,
        _ray: &Ray,
        _hit_record: &HitRecord,
        _u: Real,
        _v: Real,
        _point: Point,
    ) -> Color {
        Color::broadcast(0 as Real)
    }
}
