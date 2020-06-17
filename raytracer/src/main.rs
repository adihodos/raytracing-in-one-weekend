use std::rc::Rc;

mod camera;
mod dielectric;
mod generic_handle;
mod hittable;
mod hittable_list;
mod lambertian;
mod material;
mod metal;
mod sphere;
mod types;

use dielectric::Dielectric;
use hittable::Hittable;
use hittable_list::HittableList;
use lambertian::Lambertian;
use metal::Metal;
use sphere::Sphere;
use types::*;

fn write_pixel(
    w: &mut impl std::io::Write,
    color: Color,
    samples_per_pixel: i32,
) -> std::io::Result<()> {
    let (r, g, b) = (color * (1f32 / samples_per_pixel as f32))
        .sqrt() // gamma correct for gamma = 2.0
        .into();

    writeln!(
        w,
        "{} {} {}",
        (C_256 * clamp(r, 0f32, 0.999f32)) as i32,
        (C_256 * clamp(g, 0f32, 0.999f32)) as i32,
        (C_256 * clamp(b, 0f32, 0.999f32)) as i32
    )
}

fn write_ppm<P: AsRef<std::path::Path>>(
    file_path: P,
    width: u32,
    height: u32,
    samples_per_pixel: i32,
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
            write_pixel(&mut w, pixels[(y * width + x) as usize], samples_per_pixel)
                .expect("Failed to write image!");
        });
    });

    Ok(())
}

fn ray_color(r: &Ray, world: &HittableList, depth: i32) -> Color {
    if depth <= 0 {
        return Color::same(0f32);
    }

    if let Some(rec) = world.hit(r, 0.001f32, C_INFINITY) {
        if let Some(scatter) = rec.mtl.scatter(r, &rec) {
            return scatter.attenuation * ray_color(&scatter.ray, world, depth - 1);
        } else {
            return Color::same(0f32);
        }
    }

    use math::vec3::normalize;
    let unit_direction = normalize(r.direction);
    let t = 0.5f32 * (unit_direction.y + 1.0);

    (1f32 - t) * Color::same(1f32) + t * Color::new(0.5f32, 0.7f32, 1f32)
}

fn make_world() -> Vec<Rc<dyn Hittable>> {
    vec![
        Rc::new(Sphere::new(
            Point::new(0f32, 0f32, -1f32),
            0.5f32,
            Rc::new(Lambertian::new(Color::new(0.1f32, 0.2f32, 0.5f32))),
        )),
        Rc::new(Sphere::new(
            Point::new(0f32, -100.5f32, -1f32),
            100f32,
            Rc::new(Lambertian::new(Color::new(0.8f32, 0.8f32, 0f32))),
        )),
        Rc::new(Sphere::new(
            Point::new(1f32, 0f32, -1f32),
            0.5f32,
            Rc::new(Metal::new(Color::new(0.8f32, 0.6f32, 0.2f32), 0.3f32)),
        )),
        Rc::new(Sphere::new(
            Point::new(-1f32, 0f32, -1f32),
            0.5f32,
            Rc::new(Dielectric::new(1.5f32)),
        )),
        Rc::new(Sphere::new(
            Point::new(-1f32, 0f32, -1f32),
            -0.45f32,
            Rc::new(Dielectric::new(1.5f32)),
        )),
    ]
}

fn make_world2() -> Vec<Rc<dyn Hittable>> {
    let r = (C_PI / 4 as Real).cos();
    vec![
        Rc::new(Sphere::new(
            Point::new(-r, 0 as Real, -1 as Real),
            r,
            Rc::new(Lambertian::new(Color::new(0 as Real, 0 as Real, 1 as Real))),
        )),
        Rc::new(Sphere::new(
            Point::new(r, 0 as Real, -1 as Real),
            r,
            Rc::new(Lambertian::new(Color::new(1 as Real, 0 as Real, 0 as Real))),
        )),
    ]
}

fn main() -> std::result::Result<(), String> {
    const ASPECT_RATIO: Real = 16f32 / 9f32;
    const IMAGE_WIDTH: i32 = 384;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as i32;
    const SAMPLES_PER_PIXEL: i32 = 100;
    const MAX_DEPTH: i32 = 50;

    let mut pixels = vec![Color::same(0f32); (IMAGE_WIDTH * IMAGE_HEIGHT) as usize];

    use std::iter::FromIterator;

    let world = HittableList::from_iter(make_world2());
    let cam = camera::Camera::new(90 as Real, IMAGE_WIDTH as Real / IMAGE_HEIGHT as Real);

    (0..IMAGE_HEIGHT).rev().for_each(|y| {
        (0..IMAGE_WIDTH).for_each(|x| {
            let pixel_color = (0..SAMPLES_PER_PIXEL).fold(Color::same(0f32), |color, _| {
                let u = (x as Real + random_real()) / (IMAGE_WIDTH - 1) as f32;
                let v = (y as Real + random_real()) / (IMAGE_HEIGHT - 1) as f32;
                let r = cam.get_ray(u, v);
                color + ray_color(&r, &world, MAX_DEPTH)
            });

            pixels[(y * IMAGE_WIDTH + x) as usize] = pixel_color;
        });
    });

    write_ppm(
        "raytraced.ppm",
        IMAGE_WIDTH as u32,
        IMAGE_HEIGHT as u32,
        SAMPLES_PER_PIXEL,
        &pixels,
    )
    .map_err(|e| format!("Failed to write ppm, error {}", e))?;

    Ok(())
}
