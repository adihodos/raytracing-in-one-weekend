use crate::types::{Point, Ray, Real, Vec3};

#[derive(Copy, Clone)]
pub struct Camera {
    origin: Point,
    lower_left_corner: Point,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn new(
        lookfrom: Point,
        lookat: Point,
        world_up: Vec3,
        vertical_fov: Real,
        aspect_ratio: Real,
    ) -> Camera {
        use crate::types::degrees_to_radians;
        use math::vec3::{cross, normalize};

        let theta = degrees_to_radians(vertical_fov);
        let half_height = (theta / 2 as Real).tan();
        let half_width = aspect_ratio * half_height;

        // view direction vector
        let w = normalize(lookfrom - lookat);
        // vector to the right of the view direction
        let u = normalize(cross(world_up, w));
        // up vector for camera
        let v = cross(w, u);

        Camera {
            origin: lookfrom,
            lower_left_corner: lookfrom - v * half_height - u * half_width - w,
            horizontal: u * half_width * 2 as Real,
            vertical: v * half_height * 2 as Real,
        }
    }

    pub fn get_ray(&self, s: Real, t: Real) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin,
        )
    }
}
