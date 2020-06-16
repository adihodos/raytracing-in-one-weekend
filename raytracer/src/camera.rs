use crate::types::{Point, Ray, Real, Vec3};

#[derive(Copy, Clone)]
pub struct Camera {
    origin: Point,
    lower_left_corner: Point,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn new() -> Camera {
        const ASPECT_RATIO: Real = 16f32 / 9f32;
        const VIEWPORT_HEIGHT: Real = 2f32;
        const VIEWPORT_WIDTH: Real = VIEWPORT_HEIGHT * ASPECT_RATIO;
        const FOCAL_LENGTH: Real = 1f32;

        let origin = Vec3::same(0f32);
        let horizontal = Vec3::new(VIEWPORT_WIDTH, 0f32, 0f32);
        let vertical = Vec3::new(0f32, VIEWPORT_HEIGHT, 0f32);
        let lower_left_corner =
            origin - horizontal / 2f32 - vertical / 2f32 - Vec3::new(0f32, 0f32, FOCAL_LENGTH);

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn get_ray(&self, u: Real, v: Real) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + self.horizontal * u + self.vertical * v - self.origin,
        )
    }
}
