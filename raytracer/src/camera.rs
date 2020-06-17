use crate::types::{Point, Ray, Real, Vec3};

#[derive(Copy, Clone)]
pub struct Camera {
    origin: Point,
    lower_left_corner: Point,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    // pub fn new() -> Camera {
    //     const ASPECT_RATIO: Real = 16f32 / 9f32;
    //     const VIEWPORT_HEIGHT: Real = 2f32;
    //     const VIEWPORT_WIDTH: Real = VIEWPORT_HEIGHT * ASPECT_RATIO;
    //     const FOCAL_LENGTH: Real = 1f32;

    //     let origin = Vec3::same(0f32);
    //     let horizontal = Vec3::new(VIEWPORT_WIDTH, 0f32, 0f32);
    //     let vertical = Vec3::new(0f32, VIEWPORT_HEIGHT, 0f32);
    //     let lower_left_corner =
    //         origin - horizontal / 2f32 - vertical / 2f32 - Vec3::new(0f32, 0f32, FOCAL_LENGTH);

    //     Camera {
    //         origin,
    //         lower_left_corner,
    //         horizontal,
    //         vertical,
    //     }
    // }

    pub fn new(vertical_fov: Real, aspect_ratio: Real) -> Camera {
        use crate::types::{degrees_to_radians, C_HALF_ONE, C_ONE, C_TWO, C_ZERO};

        let theta = degrees_to_radians(vertical_fov);
        let h = (theta / 2 as Real).tan();
        let viewport_height = 2 as Real * h;
        let viewport_width = aspect_ratio * viewport_height;

        let horizontal = Vec3::new(viewport_width, 0f32, 0f32);
        let vertical = Vec3::new(0f32, viewport_height, 0f32);
        let focal_length = 1 as Real;

        Camera {
            origin: Vec3::same(0 as Real),
            lower_left_corner: Vec3::same(0 as Real)
                - C_HALF_ONE * horizontal
                - C_HALF_ONE * vertical
                - Vec3::new(C_ZERO, C_ZERO, focal_length),
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
