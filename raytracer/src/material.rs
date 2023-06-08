use crate::hittable::HitRecord;
use crate::types::{Color, Point, Ray, Real};

#[derive(Copy, Clone, Debug)]
pub struct ScatterRecord {
    pub ray: Ray,
    pub albedo: Color,
    pub pdf: Real,
}

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord>;

    fn scattering_pdf(
        &self,
        _ray: &Ray,
        _hit_record: &HitRecord,
        _scattered: &ScatterRecord,
    ) -> Real {
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
