use crate::hittable::HitRecord;
use crate::material::{Material, ScatterRecord};
use crate::types::{Color, Ray, Real};

#[derive(Copy, Clone, Debug)]
pub struct Dielectric {
    pub refraction_index: Real,
}

impl Dielectric {
    pub fn new(refraction_index: Real) -> Dielectric {
        Dielectric { refraction_index }
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        let etai_over_etat = if hit_record.front_face {
            1 as Real / self.refraction_index
        } else {
            self.refraction_index
        };

        use math::vec3::{dot, normalize, reflect_unit_vector, refract};
        let uv = normalize(ray.direction);
        let cos_theta = dot(-uv, hit_record.normal).min(1 as Real);
        let sin_theta = (1 as Real - cos_theta * cos_theta).sqrt();

        if etai_over_etat * sin_theta > 1 as Real {
            // reflect
            Some(ScatterRecord {
                ray: Ray::new(
                    hit_record.p,
                    reflect_unit_vector(uv, normalize(hit_record.normal)),
                    ray.time,
                ),
                attenuation: Color::broadcast(1 as Real),
            })
        } else {
            // schlick approximation
            use crate::types::{random_real, schlick};
            let reflect_probability = schlick(cos_theta, etai_over_etat);
            if random_real() < reflect_probability {
                Some(ScatterRecord {
                    ray: Ray::new(
                        hit_record.p,
                        reflect_unit_vector(uv, hit_record.normal),
                        ray.time,
                    ),
                    attenuation: Color::broadcast(1 as Real),
                })
            } else {
                // refract
                Some(ScatterRecord {
                    attenuation: Color::broadcast(1 as Real),
                    ray: Ray::new(
                        hit_record.p,
                        refract(uv, hit_record.normal, etai_over_etat),
                        ray.time,
                    ),
                })
            }
        }
    }
}
