use rand::Rng;

use crate::{
    sampling::{SampleStrategy, SamplerBase},
    types::{Point, Ray, Real, Vec3},
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

    pub fn get_ray(&self, s: Real, t: Real) -> Ray {
        use crate::types::random_in_unit_disk;
        let rd = self.lens_radius * random_in_unit_disk();
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
}
