use rand::Rng;
use std::sync::Arc;

use crate::{
    hittable::Hittable,
    hittable_list::HittableList,
    material::ScatterRecord,
    pdf::{HittablePdf, MixturePdf, Pdf},
    sampling::{SampleStrategy, SamplerBase},
    types::{Color, Point, Ray, Real, Vec3, C_INFINITY, C_ZERO},
    RaytracerParams,
};

#[derive(Copy, Clone)]
pub struct Camera {
    origin: Point,
    lower_left_corner: Point,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    lens_radius: Real,
    time0: Real,
    time1: Real,
}

impl Camera {
    pub fn new(
        lookfrom: Point,
        lookat: Point,
        world_up: Vec3,
        vertical_fov: Real,
        aspect_ratio: Real,
        aperture: Real,
        focus_dist: Real,
        time0: Real,
        time1: Real,
    ) -> Camera {
        use crate::types::degrees_to_radians;
        use math::vec3::{cross, normalize};

        let theta = degrees_to_radians(vertical_fov);
        let half_height = (theta / 2 as Real).tan();
        let half_width = aspect_ratio * half_height;

        //
        // view direction vector
        let w = normalize(lookfrom - lookat);
        // vector to the right of the view direction
        let u = normalize(cross(world_up, w));
        // up vector for camera
        let v = cross(w, u);

        Camera {
            origin: lookfrom,
            lower_left_corner: lookfrom
                - focus_dist * v * half_height
                - focus_dist * u * half_width
                - focus_dist * w,
            horizontal: focus_dist * u * half_width * 2 as Real,
            vertical: focus_dist * v * half_height * 2 as Real,
            lens_radius: aperture * 0.5 as Real,
            u,
            v,
            w,
            time0,
            time1,
        }
    }

    pub fn get_ray_perspective<S: SampleStrategy>(
        &self,
        s: Real,
        t: Real,
        smp: &mut SamplerBase<S>,
    ) -> Ray {
        let rd = self.lens_radius * smp.sample_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
            rand::thread_rng().gen_range(self.time0, self.time1),
        )
    }

    pub fn get_ray_ortho<S: SampleStrategy>(
        &self,
        s: Real,
        t: Real,
        smp: &mut SamplerBase<S>,
    ) -> Ray {
        let rd = self.lens_radius * smp.sample_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;

        Ray {
            origin: self.lower_left_corner + s * self.horizontal + t * self.vertical + offset,
            direction: -self.w,
            time: crate::types::random_real_range(self.time0, self.time1),
        }
    }

    pub fn raytrace_pixel<S: SampleStrategy>(
        &self,
        x: i32,
        y: i32,
        params: &RaytracerParams,
        world: &Arc<HittableList>,
        lights: &Arc<HittableList>,
        s: &mut SamplerBase<S>,
    ) -> Color {
        (0..params.samples_per_pixel).fold(Color::broadcast(0 as Real), |color, _| {
            let off = s.sample_unit_square();
            let u = (x as Real + off.x) / (params.image_width - 1) as Real;
            let v = 1 as Real - (y as Real + off.y) / (params.image_height - 1) as Real;
            let r = self.get_ray_perspective(u, v, s);
            color
                + Self::ray_color(
                    &r,
                    params.background.into(),
                    &world,
                    lights.clone(),
                    params.max_ray_depth,
                )
        })
    }

    fn ray_color(
        r: &Ray,
        background: Color,
        world: &HittableList,
        lights: Arc<dyn Hittable>,
        depth: i32,
    ) -> Color {
        if depth <= 0 {
            return Color::broadcast(C_ZERO);
        }

        if let Some(rec) = world.hit(r, 0.001 as Real, C_INFINITY) {
            let emitted = rec.mtl.emitted(r, &rec, rec.u, rec.v, rec.p);
            if let Some(scatter) = rec.mtl.scatter(r, &rec) {
                return match scatter {
                    ScatterRecord::SpecularRec { ray, attenuation } => {
                        attenuation * Self::ray_color(&ray, background, world, lights, depth - 1)
                    }
                    ScatterRecord::PdfRec { pdf, attenuation } => {
                        let light_pdf = HittablePdf {
                            obj: lights.clone(),
                            origin: rec.p,
                        };

                        let mixed_pdf = MixturePdf::new(Arc::new(light_pdf), pdf);
                        let scattered_ray = Ray::new(rec.p, mixed_pdf.generate(), r.time);
                        let pdf_val = mixed_pdf.value(scattered_ray.direction);
                        let pdf_val = if pdf_val.abs() < 1.0E-5 {
                            if pdf_val.is_sign_positive() {
                                1.0E-4
                            } else {
                                -1.0E-4
                            }
                        } else {
                            pdf_val
                        };

                        emitted
                            + attenuation
                                * rec.mtl.scattering_pdf(r, &rec, &scattered_ray)
                                * Self::ray_color(
                                    &scattered_ray,
                                    background,
                                    world,
                                    lights,
                                    depth - 1,
                                )
                                / pdf_val
                    }
                };
            } else {
                return emitted;
            }
        } else {
            return background;
        }
    }
}
