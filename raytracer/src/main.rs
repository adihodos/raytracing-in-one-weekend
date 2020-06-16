mod hittable;
mod hittable_list;
mod sphere;
mod types;

use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;
use sphere::Sphere;
use types::*;

fn write_pixel(w: &mut impl std::io::Write, color: Color) -> std::io::Result<()> {
    writeln!(
        w,
        "{} {} {}",
        (C_255_999 * color.x) as i32,
        (C_255_999 * color.y) as i32,
        (C_255_999 * color.z) as i32
    )
}

fn write_ppm<P: AsRef<std::path::Path>>(
    file_path: P,
    width: u32,
    height: u32,
    pixels: &[Color],
) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufWriter;

    let f = File::create(file_path)?;
    let mut w = BufWriter::new(f);

    write!(w, "P3\n{} {}\n255\n", width, height)?;

    (0..height).rev().for_each(|y| {
        (0..width).for_each(|x| {
            write_pixel(&mut w, pixels[(y * width + x) as usize]).expect("Failed to write image!");
        });
    });

    Ok(())
}

fn hit_sphere(center: Point, radius: Real, ray: &Ray) -> f32 {
    use math::vec3::dot;

    let oc = ray.origin - center;
    let a = ray.direction.length_squared();
    let half_b = dot(oc, ray.direction);
    // let b = 2f32 * dot(oc, ray.direction);
    let c = oc.length_squared() - radius * radius;

    let delta = half_b * half_b - a * c;
    if delta < 0f32 {
        -1f32
    } else {
        (-half_b - delta.sqrt()) / a
    }
}

fn ray_color(r: &Ray, world: &HittableList) -> Color {
    if let Some(rec) = world.hit(r, 0f32, C_INFINITY) {
        return 0.5f32 * (rec.normal + Color::same(1f32));
    }

    use math::vec3::normalize;
    let unit_direction = normalize(r.direction);
    let t = 0.5f32 * (unit_direction.y + 1.0);

    (1f32 - t) * Color::same(1f32) + t * Color::new(0.5f32, 0.7f32, 1f32)
}

fn main() -> std::result::Result<(), String> {
    const ASPECT_RATIO: Real = 16f32 / 9f32;
    const IMAGE_WIDTH: i32 = 384;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as i32;
    const VIEWPORT_HEIGHT: Real = 2f32;
    const VIEWPORT_WIDTH: Real = VIEWPORT_HEIGHT * ASPECT_RATIO;
    const FOCAL_LENGTH: Real = 1f32;

    let origin = Vec3::same(0f32);
    let horizontal = Vec3::new(VIEWPORT_WIDTH, 0f32, 0f32);
    let vertical = Vec3::new(0f32, VIEWPORT_HEIGHT, 0f32);
    let lower_left_corner =
        origin - horizontal / 2f32 - vertical / 2f32 - Vec3::new(0f32, 0f32, FOCAL_LENGTH);

    let mut pixels = vec![Color::same(0f32); (IMAGE_WIDTH * IMAGE_HEIGHT) as usize];

    use std::iter::FromIterator;
    use std::rc::Rc;

    let world_objects: Vec<Rc<dyn Hittable>> = vec![
        Rc::new(Sphere::new(Point::new(0f32, 0f32, -1f32), 0.5f32)),
        Rc::new(Sphere::new(Point::new(0f32, -100.5f32, -1f32), 100f32)),
    ];

    let world = HittableList::from_iter(world_objects);

    (0..IMAGE_HEIGHT).rev().for_each(|y| {
        (0..IMAGE_WIDTH).for_each(|x| {
            let u = x as f32 / (IMAGE_WIDTH - 1) as f32;
            let v = y as f32 / (IMAGE_HEIGHT - 1) as f32;

            let ray = Ray::new(
                origin,
                lower_left_corner + u * horizontal + v * vertical - origin,
            );
            pixels[(y * IMAGE_WIDTH + x) as usize] = ray_color(&ray, &world);
        });
    });

    write_ppm(
        "raytraced.ppm",
        IMAGE_WIDTH as u32,
        IMAGE_HEIGHT as u32,
        &pixels,
    )
    .map_err(|e| format!("Failed to write ppm, error {}", e))?;

    Ok(())
}
