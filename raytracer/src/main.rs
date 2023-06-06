#![allow(dead_code)]

use std::sync::Arc;

mod camera;
mod dielectric;
mod generic_handle;
mod hittable;
mod hittable_list;
mod lambertian;
mod material;
mod metal;
mod objects;

mod types;

use dielectric::Dielectric;
use hittable::Hittable;
use hittable_list::HittableList;
use lambertian::Lambertian;
use metal::Metal;
use objects::{disk::Disk, plane::Plane, sphere::Sphere, triangle::Triangle};

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
            let (r, g, b) = math::vec3::sqrt(*color * (1 as Real / samples_per_pixel as Real))
                // gamma correct for gamma = 2.0
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
        return Color::broadcast(0 as Real);
    }

    if let Some(rec) = world.hit(r, 0.001 as Real, C_INFINITY) {
        if let Some(scatter) = rec.mtl.scatter(r, &rec) {
            return scatter.attenuation * ray_color(&scatter.ray, world, depth - 1);
        } else {
            return Color::broadcast(0 as Real);
        }
    }

    use math::vec3::normalize;
    let unit_direction = normalize(r.direction);
    let t = 0.5 as Real * (unit_direction.y + 1 as Real);

    (1 as Real - t) * Color::broadcast(1 as Real)
        + t * Color::new(0.5 as Real, 0.7 as Real, 1 as Real)
}

fn make_random_world() -> HittableList {
    let ground_material = Arc::new(Lambertian::new(Color::broadcast(0.5 as Real)));
    let mut world = HittableList::new();

    world.add(Arc::new(Sphere::new(
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

            use math::vec3::{length, sqrt};

            if length(center - sqrt(Point::new(4 as Real, 0.2 as Real, 0 as Real))) > 0.9 as Real {
                use crate::material::Material;
                let choose_mat = random_real();

                let sphere_material: Arc<dyn Material> = if choose_mat < 0.8 as Real {
                    // diffuse
                    let albedo = random_color() * random_color();
                    Arc::new(Lambertian::new(albedo))
                } else if choose_mat < 0.95 as Real {
                    // metal
                    let albedo = random_color_in_range(0.5 as Real, 1 as Real);
                    let fuzziness = random_real_range(0 as Real, 0.5 as Real);
                    Arc::new(Metal::new(albedo, fuzziness))
                } else {
                    // glass
                    Arc::new(Dielectric::new(1.5 as Real))
                };

                world.add(Arc::new(Sphere::new(center, 0.2 as Real, sphere_material)));
            }
        });
    });

    world.add(Arc::new(Sphere::new(
        Point::new(0 as Real, 1 as Real, 0 as Real),
        1 as Real,
        Arc::new(Dielectric::new(1.5 as Real)),
    )));

    world.add(Arc::new(Sphere::new(
        Point::new(-4 as Real, 1 as Real, 0 as Real),
        1 as Real,
        Arc::new(Lambertian::new(Color::new(
            0.4 as Real,
            0.2 as Real,
            0.1 as Real,
        ))),
    )));

    world.add(Arc::new(Sphere::new(
        Point::new(4 as Real, 1 as Real, 0 as Real),
        1 as Real,
        Arc::new(Metal::new(
            Color::new(0.7 as Real, 0.6 as Real, 0.5 as Real),
            0 as Real,
        )),
    )));

    world
}

fn make_random_world2() -> HittableList {
    let ground_mtl = Arc::new(Lambertian::new(Color::new(
        0.9 as Real,
        0.3 as Real,
        0 as Real,
    )));

    let mut world = HittableList::new();
    world.add(Arc::new(Plane::new(
        Point::broadcast(0 as Real),
        Vec3::new(0 as Real, 1 as Real, 0 as Real),
        ground_mtl,
    )));

    let disk_mtl = Arc::new(Lambertian::new(Color::new(
        0 as Real,
        0.9 as Real,
        0.1 as Real,
    )));

    world.add(Arc::new(Disk {
        origin: Point::new(0 as Real, 0 as Real, -1 as Real),
        normal: Vec3::new(0 as Real, 0 as Real, 1 as Real),
        radius: 0.5 as Real,
        mtl: disk_mtl,
    }));

    let triangle_mtl = Arc::new(Metal::new(
        Color::new(0 as Real, 0.1 as Real, 0.9 as Real),
        0.1 as Real,
    ));

    world.add(Arc::new(Triangle::new(
        Point::new(-5 as Real, 0 as Real, -1 as Real),
        Point::new(5 as Real, 0 as Real, -2 as Real),
        Point::new(0 as Real, 3 as Real, -1 as Real),
        triangle_mtl,
    )));

    world
}

#[derive(Copy, Clone, Debug)]
struct RaytracerParams {
    workers: i32,
    aspect_ratio: Real,
    image_width: i32,
    image_height: i32,
    samples_per_pixel: i32,
    max_ray_depth: i32,
    vertical_fov: Real,
    look_from: Point,
    look_at: Point,
    world_up: Vec3,
    aperture: Real,
    focus_dist: Real,
}

#[derive(Copy, Clone, Debug)]
struct WorkBlock {
    xdim: (i32, i32),
    ydim: (i32, i32),
}

#[derive(Copy, Clone, Debug)]
struct ImageOutput {
    pixels: *mut Color,
    width: i32,
    height: i32,
}

unsafe impl std::marker::Send for ImageOutput {}
unsafe impl std::marker::Sync for ImageOutput {}

fn raytrace_mt(params: RaytracerParams, world: HittableList) -> Vec<Color> {
    const WORKER_BLOCK_DIM: i32 = 16;

    let blocks_x = (params.image_width / WORKER_BLOCK_DIM) + 1;
    let blocks_y = (params.image_height / WORKER_BLOCK_DIM) + 1;

    let mut workblocks = vec![];
    (0..blocks_y).for_each(|yblk| {
        (0..blocks_x).for_each(|xblk| {
            workblocks.push(WorkBlock {
                xdim: (
                    (xblk * WORKER_BLOCK_DIM).min(params.image_width),
                    ((xblk + 1) * WORKER_BLOCK_DIM).min(params.image_width),
                ),
                ydim: (
                    (yblk * WORKER_BLOCK_DIM).min(params.image_height),
                    ((yblk + 1) * WORKER_BLOCK_DIM).min(params.image_height),
                ),
            });
        });
    });

    let cam = camera::Camera::new(
        params.look_from,
        params.look_at,
        params.world_up,
        params.vertical_fov,
        params.aspect_ratio,
        params.aperture,
        params.focus_dist,
    );

    let total_workblocks = workblocks.len();
    let world = Arc::new(world);
    use std::sync::Mutex;
    let workblocks = Arc::new(Mutex::new(workblocks));
    let mut image_pixels =
        vec![Color::broadcast(0 as Real); (params.image_width * params.image_height) as usize];

    let workblocks_done = Arc::new(std::sync::atomic::AtomicI32::new(0));

    let workers = (0..params.workers)
        .map(|worker_idx| {
            let workblocks = Arc::clone(&workblocks);
            let world = Arc::clone(&world);
            let output_pixels = ImageOutput {
                pixels: image_pixels.as_mut_ptr(),
                width: params.image_width,
                height: params.image_height,
            };
            let workblocks_done = Arc::clone(&workblocks_done);

            std::thread::spawn(move || loop {
                //
                // pop a work package from the queue
                let maybe_this_work_pkg = if let Ok(ref mut work_queue) = workblocks.lock() {
                    work_queue.pop()
                } else {
                    None
                };

                if let Some(this_work_pkg) = maybe_this_work_pkg {
                    //
                    // process pixels in this work package
                    (this_work_pkg.ydim.0..this_work_pkg.ydim.1)
                        .rev()
                        .for_each(|y| {
                            (this_work_pkg.xdim.0..this_work_pkg.xdim.1).for_each(|x| {
                                //
                                // Raytrace this pixel
                                let pixel_color = (0..params.samples_per_pixel).fold(
                                    Color::broadcast(0 as Real),
                                    |color, _| {
                                        let u = (x as Real + random_real())
                                            / (params.image_width - 1) as Real;
                                        let v = 1 as Real
                                            - (y as Real + random_real())
                                                / (params.image_height - 1) as Real;
                                        let r = cam.get_ray(u, v);
                                        color + ray_color(&r, &world, params.max_ray_depth)
                                    },
                                );

                                unsafe {
                                    output_pixels
                                        .pixels
                                        .add((y * params.image_width + x) as usize)
                                        .write(pixel_color);
                                }
                            });
                        });

                    workblocks_done.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                } else {
                    println!(
                        "No more work or queue locking failure, worker {} quitting ...",
                        worker_idx
                    );
                    break;
                }
            })
        })
        .collect::<Vec<_>>();

    loop {
        let processed = workblocks_done.load(std::sync::atomic::Ordering::SeqCst);
        println!(
            "Processed {} workblocks out of {} total",
            processed, total_workblocks
        );

        if processed == total_workblocks as i32 {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    workers
        .into_iter()
        .for_each(|w| w.join().expect("Failed to join worker!"));

    image_pixels
}

fn main() -> std::result::Result<(), String> {
    // let v = Vec3::new(1 as Real, 2 as Real, 0.5 as Real);
    let v0 = Vec3::broadcast(-1f32);
    let v1 = Vec3::broadcast(2f32);
    let v = math::vec3::min(v0, v1);

    type IVec3 = math::vec3::TVec3<i32>;
    let v2 = IVec3::broadcast(1);
    let v3 = IVec3::broadcast(4);
    let v = math::vec3::min(v2, v3);
    // let cv = math::vec3::cos(v);
    println!("Floating point precision: {}", FP_MODEL);
    // const ASPECT_RATIO: Real = 16f32 / 9f32;
    const IMAGE_WIDTH: i32 = 1200;
    const IMAGE_HEIGHT: i32 = 800;
    const SAMPLES_PER_PIXEL: i32 = 256;
    const MAX_DEPTH: i32 = 50;

    let look_from = Point::new(13 as Real, 2 as Real, 3 as Real);
    let look_at = Point::new(0 as Real, 0 as Real, 0 as Real);
    let world_up = Vec3::new(0 as Real, 1 as Real, 0 as Real);
    let aperture = 0.1 as Real;
    let focus_dist = 10 as Real;
    let vertical_fov = 20 as Real;

    let params = RaytracerParams {
        workers: 8,
        image_width: IMAGE_WIDTH,
        image_height: IMAGE_HEIGHT,
        samples_per_pixel: SAMPLES_PER_PIXEL,
        max_ray_depth: MAX_DEPTH,
        aspect_ratio: (IMAGE_WIDTH as Real) / (IMAGE_HEIGHT as Real),
        look_at,
        look_from,
        world_up,
        aperture,
        focus_dist,
        vertical_fov,
    };

    let start_time = std::time::Instant::now();
    let raytraced_pixels = raytrace_mt(params, make_random_world());
    let render_duration = start_time.elapsed();

    let minutes = render_duration.as_secs() / 60u64;
    let seconds = (render_duration - std::time::Duration::from_secs(minutes * 60u64)).as_secs();

    println!(
        "Finished! Rendering settings : floating point model: {} precision\n{:?}\nTotal render time = {} minutes {} seconds",
        FP_MODEL,
        params, minutes, seconds
    );

    write_png(
        "raytraced_mt.png",
        params.image_width as u32,
        params.image_height as u32,
        params.samples_per_pixel,
        &raytraced_pixels,
    )
    .map_err(|e| format!("Failed to write image, error {}", e))?;

    Ok(())
}
