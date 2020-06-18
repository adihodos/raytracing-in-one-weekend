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

fn write_png<P: AsRef<std::path::Path>>(
    file_path: P,
    img_width: u32,
    img_height: u32,
    samples_per_pixel: i32,
    pixels: &[Color],
) -> std::io::Result<()> {
    //
    // gamma correct anb transform to 8bpp color
    let pixels_rgb = pixels
        .iter()
        .map(|color| {
            let (r, g, b) = (*color * (1 as Real / samples_per_pixel as Real))
                .sqrt() // gamma correct for gamma = 2.0
                .into();

            const COLOR_CLAMP_MIN: Real = 0 as Real;
            const COLOR_CLAMP_MAX: Real = 0.999 as Real;

            (
                (256 as Real * clamp(r, COLOR_CLAMP_MIN, COLOR_CLAMP_MAX)) as u8,
                (256 as Real * clamp(g, COLOR_CLAMP_MIN, COLOR_CLAMP_MAX)) as u8,
                (256 as Real * clamp(b, COLOR_CLAMP_MIN, COLOR_CLAMP_MAX)) as u8,
            )
        })
        .collect::<Vec<_>>();

    use std::fs::File;
    use std::io::BufWriter;

    let out_file = File::create(file_path)?;
    let ref mut file_writer = BufWriter::new(out_file);
    let mut png_encoder = png::Encoder::new(file_writer, img_width, img_height);
    png_encoder.set_color(png::ColorType::RGB);
    png_encoder.set_depth(png::BitDepth::Eight);

    png_encoder
        .write_header()
        .and_then(|mut png_writer| {
            png_writer.write_image_data(unsafe {
                std::slice::from_raw_parts(pixels_rgb.as_ptr() as *const u8, pixels_rgb.len() * 3)
            })
        })
        .map_err(|encoding_err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("PNG encoding error: {}", encoding_err),
            )
        })
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
            Rc::new(Metal::new(Color::new(0.8f32, 0.6f32, 0.2f32), 0f32)),
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

fn make_random_world() -> HittableList {
    let ground_material = Rc::new(Lambertian::new(Color::same(0.5f32)));
    let mut world = HittableList::new();

    world.add(Rc::new(Sphere::new(
        Point::new(0 as Real, -1000 as Real, 0 as Real),
        1000 as Real,
        ground_material,
    )));

    (-11..11).for_each(|a| {
        (-11..11).for_each(|b| {
            let center = Point::new(
                a as Real + 0.9 as Real + random_real(),
                0.2 as Real,
                b as Real + 0.9 as Real * random_real(),
            );

            if (center - Point::new(4 as Real, 0.2 as Real, 0 as Real)).length() > 0.9 as Real {
                use crate::material::Material;
                let choose_mat = random_real();

                let sphere_material: Rc<dyn Material> = if choose_mat < 0.8 as Real {
                    // diffuse
                    let albedo = random_color() * random_color();
                    Rc::new(Lambertian::new(albedo))
                } else if choose_mat < 0.95 as Real {
                    // metal
                    let albedo = random_color_in_range(0.5 as Real, 1 as Real);
                    let fuzziness = random_real_range(0 as Real, 0.5 as Real);
                    Rc::new(Metal::new(albedo, fuzziness))
                } else {
                    // glass
                    Rc::new(Dielectric::new(1.5 as Real))
                };

                world.add(Rc::new(Sphere::new(center, 0.2 as Real, sphere_material)));
            }
        });
    });

    world.add(Rc::new(Sphere::new(
        Point::new(0 as Real, 1 as Real, 0 as Real),
        1 as Real,
        Rc::new(Dielectric::new(1.5 as Real)),
    )));

    world.add(Rc::new(Sphere::new(
        Point::new(-4 as Real, 1 as Real, 0 as Real),
        1 as Real,
        Rc::new(Lambertian::new(Color::new(
            0.4 as Real,
            0.2 as Real,
            0.1 as Real,
        ))),
    )));

    world.add(Rc::new(Sphere::new(
        Point::new(4 as Real, 1 as Real, 0 as Real),
        1 as Real,
        Rc::new(Metal::new(
            Color::new(0.7 as Real, 0.6 as Real, 0.5 as Real),
            0 as Real,
        )),
    )));

    world
}

fn main() -> std::result::Result<(), String> {
    const ASPECT_RATIO: Real = 16f32 / 9f32;
    const IMAGE_WIDTH: i32 = 1200;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as i32;
    const SAMPLES_PER_PIXEL: i32 = 128;
    const MAX_DEPTH: i32 = 50;

    let world = make_random_world();

    let look_from = Point::new(13 as Real, 2 as Real, 3 as Real);
    let look_at = Point::new(0 as Real, 0 as Real, 0 as Real);
    let world_up = Vec3::new(0 as Real, 1 as Real, 0 as Real);
    let aperture = 0.1 as Real;
    let focus_dist = 10 as Real;

    let cam = camera::Camera::new(
        look_from,
        look_at,
        world_up,
        20 as Real,
        IMAGE_WIDTH as Real / IMAGE_HEIGHT as Real,
        aperture,
        focus_dist,
    );

    let mut pixels = vec![Color::same(0f32); (IMAGE_WIDTH * IMAGE_HEIGHT) as usize];

    let start_time = std::time::Instant::now();
    (0..IMAGE_HEIGHT).rev().for_each(|y| {
        println!("Scanlines remaining: {}", y);
        (0..IMAGE_WIDTH).for_each(|x| {
            let pixel_color = (0..SAMPLES_PER_PIXEL).fold(Color::same(0f32), |color, _| {
                let u = (x as Real + random_real()) / (IMAGE_WIDTH - 1) as f32;
                let v = 1 as Real - (y as Real + random_real()) / (IMAGE_HEIGHT - 1) as f32;
                let r = cam.get_ray(u, v);
                color + ray_color(&r, &world, MAX_DEPTH)
            });

            pixels[(y * IMAGE_WIDTH + x) as usize] = pixel_color;
        });
    });

    let render_duration = start_time.elapsed();

    let minutes = render_duration.as_secs() / 60u64;
    let seconds = (render_duration - std::time::Duration::from_secs(minutes * 60u64)).as_secs();

    println!(
        "Finished! Total render time = {} minutes {} seconds\nWriting rendered image ...",
        minutes, seconds
    );

    write_png(
        "raytraced.png",
        IMAGE_WIDTH as u32,
        IMAGE_HEIGHT as u32,
        SAMPLES_PER_PIXEL,
        &pixels,
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
