use std::sync::Arc;

use crate::{
    hittable::Hittable,
    onb::Onb,
    types::{Point, Real, Vec3},
};

pub trait Pdf {
    fn value(&self, direction: Vec3) -> Real;
    fn generate(&self) -> Vec3;
}

#[derive(Copy, Clone, Debug)]
pub struct CosinePdf {
    pub uvw: Onb,
}

impl std::convert::From<Vec3> for CosinePdf {
    fn from(w: Vec3) -> Self {
        Self { uvw: w.into() }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: Vec3) -> Real {
        let cosine = math::vec3::dot(math::vec3::normalize(direction), self.uvw.w());

        if cosine <= (0 as Real) {
            0 as Real
        } else {
            cosine * (std::f64::consts::FRAC_1_PI as Real)
        }
    }

    fn generate(&self) -> Vec3 {
        self.uvw
            .local_from_vec(crate::types::random_cosine_direction())
    }
}

pub struct HittablePdf {
    pub origin: Point,
    pub obj: Arc<dyn Hittable>,
}

impl Pdf for HittablePdf {
    fn value(&self, direction: Vec3) -> Real {
        self.obj.pdf_value(self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.obj.random(self.origin)
    }
}
