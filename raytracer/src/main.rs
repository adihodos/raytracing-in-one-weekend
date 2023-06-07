#![allow(dead_code)]

use std::{
    iter::FromIterator,
    os::raw::c_void,
    sync::{mpsc::Receiver, Arc},
};

use checker_texture::CheckerTexture;
use diffuse_light::DiffuseLight;
use image_texture::ImageTexture;
use noise_texture::NoiseTexture;
use rectangles::XYRect;
use serde::{Deserialize, Serialize};

mod ui;

mod aabb3;
mod bvh;
mod camera;
mod checker_texture;
mod dielectric;
mod diffuse_light;
mod generic_handle;
mod hittable;
mod hittable_list;
mod image_texture;
mod lambertian;
mod material;
mod metal;
mod noise_texture;
mod objects;
mod perlin;
mod rectangles;
mod solid_color_texture;
mod texture;

mod types;

use dielectric::Dielectric;
use hittable::Hittable;
use hittable_list::HittableList;
use lambertian::Lambertian;
use metal::Metal;
use objects::sphere::Sphere;

use rand::seq::SliceRandom;
use rendering::gl;
use types::*;

use glfw::Context;
use ui::UiBackend;

use crate::{
    objects::sphere::MovingSphere,
    rectangles::{XZRect, YZRect},
};

const COLOR_CLAMP_MIN: Real = 0 as Real;
const COLOR_CLAMP_MAX: Real = 0.999 as Real;

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

fn ray_color(r: &Ray, background: Color, world: &HittableList, depth: i32) -> Color {
    if depth <= 0 {
        return Color::broadcast(0 as Real);
    }

    if let Some(rec) = world.hit(r, 0.001 as Real, C_INFINITY) {
        let emitted = rec.mtl.emitted(rec.u, rec.v, rec.p);
        if let Some(scatter) = rec.mtl.scatter(r, &rec) {
            return emitted
                + scatter.attenuation * ray_color(&scatter.ray, background, world, depth - 1);
        } else {
            return emitted;
        }
    } else {
        return background;
    }
}

fn scene_random_world() -> HittableList {
    let ground_material = Arc::new(Lambertian::from_texture(Arc::new(
        CheckerTexture::from_colors((0.2f32, 0.3f32, 0.1f32), (0.9f32, 0.9f32, 0.9f32)),
    )));
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
                let choose_mat = random_real();

                if choose_mat < 0.8 as Real {
                    //
                    // diffuse
                    let albedo = random_color() * random_color();
                    let sphere_material = Arc::new(Lambertian::new(albedo));
                    let center2 = center + Vec3::new(0f32, random_real_range(0f32, 0.5f32), 0f32);
                    world.add(Arc::new(MovingSphere::new(
                        center,
                        center2,
                        0f32,
                        1f32,
                        0.2 as Real,
                        sphere_material,
                    )));
                } else if choose_mat < 0.95 as Real {
                    //
                    // metal
                    let albedo = random_color_in_range(0.5 as Real, 1 as Real);
                    let fuzziness = random_real_range(0 as Real, 0.5 as Real);
                    let sphere_material = Arc::new(Metal::new(albedo, fuzziness));
                    world.add(Arc::new(Sphere::new(center, 0.2 as Real, sphere_material)));
                } else {
                    //
                    // glass
                    let sphere_material = Arc::new(Dielectric::new(1.5 as Real));
                    world.add(Arc::new(Sphere::new(center, 0.2 as Real, sphere_material)));
                }
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

fn scene_two_spheres() -> HittableList {
    let checker_mtl = Arc::new(Lambertian::from_texture(Arc::new(
        CheckerTexture::from_colors((0.2f32, 0.3f32, 0.1f32), (0.9f32, 0.9f32, 0.9f32)),
    )));

    let mut world = HittableList::new();

    world.add(Arc::new(Sphere::new(
        Point::new(0f32, -10f32, 0f32),
        10f32,
        checker_mtl.clone(),
    )));
    world.add(Arc::new(Sphere::new(
        Point::new(0f32, 10f32, 0f32),
        10f32,
        checker_mtl.clone(),
    )));

    world
}

fn scene_two_perlin_spheres() -> HittableList {
    // let image_texture = Arc::new(Lambertian::from_texture(Arc::new(ImageTexture::new(
    //     "data/textures/uv_grids/ash_uvgrid10.jpg",
    // ))));

    // let image_texture1 = Arc::new(Lambertian::from_texture(Arc::new(ImageTexture::new(
    //     "data/textures/uv_grids/ash_uvgrid01.jpg",
    // ))));

    let noise_mtl = Arc::new(Lambertian::from_texture(Arc::new(NoiseTexture::new(3f32))));

    let mut world = HittableList::new();

    world.add(Arc::new(Sphere::new(
        Point::new(0f32, -1000f32, 0f32),
        1000f32,
        noise_mtl.clone(),
    )));
    world.add(Arc::new(Sphere::new(
        Point::new(0f32, 2f32, 0f32),
        2f32,
        noise_mtl.clone(),
    )));

    world
}

fn scene_textured_spheres() -> HittableList {
    let image_texture = Arc::new(Lambertian::from_texture(Arc::new(ImageTexture::new(
        // "data/textures/uv_grids/ash_uvgrid01.jpg",
        "data/textures/misc/earthmap.jpg",
    ))));

    let mut world = HittableList::new();

    world.add(Arc::new(Sphere::new(
        Point::new(0f32, 0f32, 0f32),
        2f32,
        image_texture.clone(),
    )));

    world
}

fn scene_simple_light() -> HittableList {
    let noise_mtl = Arc::new(Lambertian::from_texture(Arc::new(NoiseTexture::new(3f32))));

    let mut world = HittableList::new();

    world.add(Arc::new(Sphere::new(
        Point::new(0f32, -1000f32, 0f32),
        1000f32,
        noise_mtl.clone(),
    )));
    world.add(Arc::new(Sphere::new(
        Point::new(0f32, 2f32, 0f32),
        2f32,
        noise_mtl.clone(),
    )));

    let diffuse_light = Arc::new(DiffuseLight::with_color((4f32, 4f32, 4f32)));
    world.add(Arc::new(XYRect {
        x0: 3f32,
        x1: 5f32,
        y0: 1f32,
        y1: 3f32,
        k: -2f32,
        mtl: diffuse_light,
    }));

    let red_light = Arc::new(DiffuseLight::with_color((4f32, 2f32, 0f32)));
    world.add(Arc::new(Sphere::new(
        Point::new(0f32, 8f32, 0f32),
        2f32,
        red_light,
    )));

    world
}

fn scene_cornell_box() -> HittableList {
    let colors = [
        (0.65f32, 0.05f32, 0.05f32),
        (0.73f32, 0.73f32, 0.73f32),
        (0.12f32, 0.45f32, 0.15f32),
    ]
    .iter()
    .map(|color| Arc::new(Lambertian::new((*color).into())))
    .collect::<Vec<_>>();

    let light = Arc::new(DiffuseLight::with_color((15f32, 15f32, 15f32)));

    enum WallType {
        XZ,
        YZ,
        XY,
    }
    struct WallData {
        wt: WallType,
        a: f32,
        b: f32,
        c: f32,
        d: f32,
        k: f32,
        color_id: usize,
    }

    let mut world: HittableList = HittableList::from_iter(
        [
            WallData {
                wt: WallType::YZ,
                a: 0f32,
                b: 555f32,
                c: 0f32,
                d: 555f32,
                k: 555f32,
                color_id: 2,
            },
            //
            // yz_rect>(0, 555, 0, 555, 0, red)
            WallData {
                wt: WallType::YZ,
                a: 0f32,
                b: 555f32,
                c: 0f32,
                d: 555f32,
                k: 0f32,
                color_id: 0,
            },
            //
            // xz_rect>(0, 555, 0, 555, 0, white)
            WallData {
                wt: WallType::XZ,
                a: 0f32,
                b: 555f32,
                c: 0f32,
                d: 555f32,
                k: 0f32,
                color_id: 1,
            },
            //
            // xz_rect>(0, 555, 0, 555, 555, white)
            WallData {
                wt: WallType::XZ,
                a: 0f32,
                b: 555f32,
                c: 0f32,
                d: 555f32,
                k: 555f32,
                color_id: 1,
            },
            //
            // xy_rect>(0, 555, 0, 555, 555, white)
            WallData {
                wt: WallType::XY,
                a: 0f32,
                b: 555f32,
                c: 0f32,
                d: 555f32,
                k: 555f32,
                color_id: 1,
            },
        ]
        .iter()
        .map(|wd| -> Arc<dyn Hittable> {
            match wd.wt {
                WallType::XY => Arc::new(XYRect {
                    x0: wd.a,
                    x1: wd.b,
                    y0: wd.c,
                    y1: wd.d,
                    k: wd.k,
                    mtl: colors[wd.color_id].clone(),
                }),
                WallType::XZ => Arc::new(XZRect {
                    x0: wd.a,
                    x1: wd.b,
                    z0: wd.c,
                    z1: wd.d,
                    k: wd.k,
                    mtl: colors[wd.color_id].clone(),
                }),

                WallType::YZ => Arc::new(YZRect {
                    y0: wd.a,
                    y1: wd.b,
                    z0: wd.c,
                    z1: wd.d,
                    k: wd.k,
                    mtl: colors[wd.color_id].clone(),
                }),
            }
        }),
    );

    world.add(Arc::new(XZRect {
        x0: 213f32,
        x1: 343f32,
        z0: 227f32,
        z1: 332f32,
        k: 554f32,
        mtl: light,
    }));

    world
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct RaytracerParams {
    workers: i32,
    worker_block_pixels: i32,
    aspect_ratio: Real,
    image_width: i32,
    image_height: i32,
    samples_per_pixel: i32,
    max_ray_depth: i32,
    vertical_fov: Real,
    look_from: [Real; 3],
    look_at: [Real; 3],
    world_up: [Real; 3],
    aperture: Real,
    focus_dist: Real,
    shuffle_workblocks: bool,
    background: [Real; 3],
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

struct RaytracerState {
    params: RaytracerParams,
    workers: Vec<std::thread::JoinHandle<()>>,
    workblocks_done: std::sync::Arc<std::sync::atomic::AtomicI32>,
    total_workblocks: u32,
    image_pixels: Vec<Color>,
    cancel_token: Arc<std::sync::atomic::AtomicBool>,
    timestamp: std::time::Instant,
    raytracing_time: std::time::Duration,
}

impl std::ops::Drop for RaytracerState {
    fn drop(&mut self) {
        let mut workers = Vec::new();
        std::mem::swap(&mut self.workers, &mut workers);
        workers.into_iter().for_each(|w| {
            w.join().expect("Failed to join worker!");
        });
    }
}

impl RaytracerState {
    fn load_parameters() -> RaytracerParams {
        let f = std::fs::File::open("data/config/raytracer.config.ron")
            .expect("Failed to open config file");

        ron::de::from_reader(f).expect("Failed to decode config file")
    }

    fn new() -> RaytracerState {
        let params = Self::load_parameters();

        let blocks_x = (params.image_width / params.worker_block_pixels) + 1;
        let blocks_y = (params.image_height / params.worker_block_pixels) + 1;

        let mut workblocks = vec![];
        (0..blocks_y).for_each(|yblk| {
            (0..blocks_x).for_each(|xblk| {
                workblocks.push(WorkBlock {
                    xdim: (
                        (xblk * params.worker_block_pixels).min(params.image_width),
                        ((xblk + 1) * params.worker_block_pixels).min(params.image_width),
                    ),
                    ydim: (
                        (yblk * params.worker_block_pixels).min(params.image_height),
                        ((yblk + 1) * params.worker_block_pixels).min(params.image_height),
                    ),
                });
            });
        });

        if params.shuffle_workblocks {
            workblocks.shuffle(&mut rand::thread_rng());
        }

        let cam = camera::Camera::new(
            params.look_from.into(),
            params.look_at.into(),
            params.world_up.into(),
            params.vertical_fov,
            params.aspect_ratio,
            params.aperture,
            params.focus_dist,
            0f32,
            1f32,
        );

        let total_workblocks = workblocks.len() as u32;
        let world = Arc::new(scene_cornell_box());
        use std::sync::Mutex;
        let workblocks = Arc::new(Mutex::new(workblocks));
        let mut image_pixels =
            vec![Color::broadcast(0 as Real); (params.image_width * params.image_height) as usize];

        let workblocks_done = Arc::new(std::sync::atomic::AtomicI32::new(0));
        let cancel_token = Arc::new(std::sync::atomic::AtomicBool::new(false));

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
                let cancel_token = Arc::clone(&cancel_token);

                std::thread::spawn(move || loop {
                    if cancel_token.load(std::sync::atomic::Ordering::SeqCst) {
                        println!("Worker {} cancelled", worker_idx);
                        break;
                    }
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
                                            color
                                                + ray_color(
                                                    &r,
                                                    params.background.into(),
                                                    &world,
                                                    params.max_ray_depth,
                                                )
                                        },
                                    );

                                    //
                                    // gamma correct
                                    let pixel_color = math::vec3::clamp(
                                        math::vec3::sqrt(
                                            pixel_color
                                                * (1 as Real / params.samples_per_pixel as Real),
                                        ),
                                        Vec3::broadcast(COLOR_CLAMP_MIN),
                                        Vec3::broadcast(COLOR_CLAMP_MAX),
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

        RaytracerState {
            total_workblocks,
            params,
            workers,
            workblocks_done,
            image_pixels,
            cancel_token,
            timestamp: std::time::Instant::now(),
            raytracing_time: std::time::Duration::from_millis(0),
        }
    }

    fn get_image_pixels(&self) -> &[f32] {
        unsafe {
            std::slice::from_raw_parts(
                self.image_pixels.as_ptr() as *const f32,
                self.image_pixels.len() * 3,
            )
        }
    }

    fn raytracing_finished(&mut self) -> bool {
        let is_finished = self
            .workblocks_done
            .load(std::sync::atomic::Ordering::SeqCst)
            > self.total_workblocks as i32;

        if self
            .workblocks_done
            .load(std::sync::atomic::Ordering::SeqCst)
            == self.total_workblocks as i32
        {
            self.workblocks_done
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        is_finished
    }

    fn cancel_work(&mut self) {
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

struct MainWindow {
    raytracer: RaytracerState,
    rtgl: RaytracingGlState,
    ui: UiBackend,
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
}

impl MainWindow {
    fn new() -> MainWindow {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to initialize GLFW");

        use glfw::WindowHint;
        glfw.window_hint(WindowHint::DoubleBuffer(true));
        glfw.window_hint(WindowHint::OpenGlDebugContext(true));
        glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(WindowHint::ClientApi(glfw::ClientApiHint::OpenGl));
        glfw.window_hint(WindowHint::ContextVersion(4, 6));
        glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        let (mut window, events) = glfw
            .create_window(
                1600,
                1200,
                "Raytracing in 1 weekend",
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create window");

        window.set_all_polling(true);
        window.make_current();

        rendering::gl::load_with(|s| window.get_proc_address(s) as *const _);

        let ui = UiBackend::new(&window);
        let raytracer = RaytracerState::new();
        let rtgl = RaytracingGlState::new(
            raytracer.params.image_width as u32,
            raytracer.params.image_height as u32,
        );

        MainWindow {
            ui,
            raytracer,
            rtgl,
            glfw,
            window,
            events,
        }
    }

    fn main_loop(&mut self) {
        while !self.window.should_close() {
            self.glfw.poll_events();

            while let Ok((_, event)) = self.events.try_recv() {
                self.handle_window_event(event);
            }

            self.update_loop();
            self.window.swap_buffers();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    fn handle_window_event(&mut self, event: glfw::WindowEvent) {
        use glfw::WindowEvent;

        match event {
            WindowEvent::Close => {
                self.window.set_should_close(true);
                self.raytracer.cancel_work();
            }

            _ => {
                self.ui.event_handler(&self.window, event);
            }
        }
    }

    fn draw_ui(&mut self) {
        let ui = self.ui.new_frame(&self.window);
        let p = self.raytracer.params;
        let work_done = self
            .raytracer
            .workblocks_done
            .load(std::sync::atomic::Ordering::SeqCst);
        let total_work = self.raytracer.total_workblocks;
        let elapsed = self.raytracer.raytracing_time;

        ui.window("Raytracer status")
            .size([400f32, 600f32], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Image size: {}x{}", p.image_width, p.image_height));
                ui.text(format!("Aspect ratio: {}", p.aspect_ratio));
                ui.text(format!("Aperture: {}", p.aperture));
                ui.text(format!("Focus distance: {}", p.focus_dist));
                ui.text(format!("Maximum ray depth: {}", p.max_ray_depth));
                ui.text(format!("Samples per pixel: {}", p.samples_per_pixel));
                ui.text(format!("Field of view: {}", p.vertical_fov));

                ui.separator();
                ui.text(format!("Worker threads: {}", p.workers));
                ui.text(format!("Randomized workloads: {}", p.shuffle_workblocks));
                imgui::ProgressBar::new(work_done as f32 / total_work as f32)
                    .overlay_text(format!(
                        "Pixel blocks raytraced {}/{}",
                        work_done, total_work
                    ))
                    .build(&ui);

                ui.text_colored(
                    [1f32, 0f32, 0f32, 1f32],
                    format!("Time spent: {}", humantime::format_duration(elapsed)),
                );
            });
    }

    fn update_loop(&mut self) {
        let (width, height) = self.window.get_framebuffer_size();

        unsafe {
            gl::ClearNamedFramebufferfv(0, gl::COLOR, 0, [0f32, 1f32, 0f32, 1f32].as_ptr());
            gl::ViewportIndexedf(0, 0f32, 0f32, width as f32, height as f32);
        }

        let frame_context = FrameRenderContext {
            framebuffer_width: width,
            framebuffer_height: height,
        };

        if !self.raytracer.raytracing_finished() {
            let current_timestamp = std::time::Instant::now();
            self.raytracer.raytracing_time += current_timestamp - self.raytracer.timestamp;
            self.raytracer.timestamp = current_timestamp;

            self.rtgl.update_texture(self.raytracer.get_image_pixels());
        }
        self.rtgl.render(&frame_context);

        //
        // render ui
        self.draw_ui();
        self.ui.render();
    }
}

fn main() -> std::result::Result<(), String> {
    let mut main_window = MainWindow::new();
    main_window.main_loop();

    Ok(())
}

#[derive(Copy, Clone, Debug)]
struct FrameRenderContext {
    framebuffer_width: i32,
    framebuffer_height: i32,
}

struct RaytracingGlState {
    vao: rendering::UniqueVertexArray,
    vs: rendering::UniqueShaderProgram,
    fs: rendering::UniqueShaderProgram,
    pipeline: rendering::UniquePipeline,
    texture: rendering::UniqueTexture,
    sampler: rendering::UniqueSampler,
    img_width: i32,
    img_height: i32,
}

impl RaytracingGlState {
    const VS_PROGRAM: &'static str = r#"
#version 460 core

layout(location = 1) out vec2 texCoord;

out gl_PerVertex {
    layout(location = 0) vec4 gl_Position;
};

void main()
{
    vec2 position = vec2(gl_VertexID % 2, gl_VertexID / 2) * 4.0 - 1;
    texCoord = (position + 1) * 0.5;
    texCoord.y = 1.0 - texCoord.y;

    gl_Position = vec4(position, 0, 1);
}
"#;

    const FS_PROGRAM: &'static str = r#"
#version 460 core

layout(location = 1) in vec2 texCoord;
layout(binding = 0) uniform sampler2D texImg;
layout(location = 0) out vec4 FinalFragColor;

void main()
{
    FinalFragColor = texture(texImg, texCoord);
}
"#;

    fn new(img_width: u32, img_height: u32) -> RaytracingGlState {
        let vao = rendering::UniqueVertexArray::new(unsafe {
            let mut vao: u32 = 0;
            gl::CreateVertexArrays(1, &mut vao as *mut _);
            vao
        })
        .expect("Failed to create vertexarray object");

        let vs = rendering::create_shader_program_from_string(
            Self::VS_PROGRAM,
            rendering::ShaderType::Vertex,
        )
        .expect("Failed to create vertex shader");

        let fs = rendering::create_shader_program_from_string(
            Self::FS_PROGRAM,
            rendering::ShaderType::Fragment,
        )
        .expect("Failed to create fragment shader");

        let pipeline = rendering::UniquePipeline::new(unsafe {
            let mut pipeline = 0u32;
            gl::GenProgramPipelines(1, &mut pipeline as *mut _);
            pipeline
        })
        .expect("Failed to create pipeline");

        unsafe {
            gl::UseProgramStages(*pipeline, gl::VERTEX_SHADER_BIT, *vs);
            gl::UseProgramStages(*pipeline, gl::FRAGMENT_SHADER_BIT, *fs);
        }

        let texture = rendering::UniqueTexture::new(unsafe {
            let mut texture = 0u32;
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture as *mut _);
            gl::TextureStorage2D(texture, 1, gl::RGB32F, img_width as i32, img_height as i32);

            texture
        })
        .expect("Failed to create texture");

        let mut text_pixels = vec![0f32; (img_width * img_height * 3) as usize];
        for y in 0..img_height {
            for x in 0..img_width {
                text_pixels[(y * img_width * 3 + x * 3 + 0) as usize] = x as f32 / img_width as f32;
                text_pixels[(y * img_width * 3 + x * 3 + 1) as usize] =
                    y as f32 / img_height as f32;
            }
        }

        unsafe {
            gl::TextureSubImage2D(
                *texture,
                0,
                0,
                0,
                img_width as i32,
                img_height as i32,
                gl::RGB,
                gl::FLOAT,
                text_pixels.as_ptr() as *const _ as *const c_void,
            );
        }

        let sampler = rendering::UniqueSampler::new(unsafe {
            let mut sampler = 0u32;
            gl::CreateSamplers(1, &mut sampler as *mut _);
            gl::SamplerParameteri(sampler, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::SamplerParameteri(
                sampler,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );

            sampler
        })
        .expect("Failed to create sampler");

        RaytracingGlState {
            vao,
            vs,
            fs,
            pipeline,
            texture,
            sampler,
            img_width: img_width as i32,
            img_height: img_height as i32,
        }
    }

    fn update_texture(&self, pixels: &[f32]) {
        unsafe {
            gl::TextureSubImage2D(
                *self.texture,
                0,
                0,
                0,
                self.img_width,
                self.img_height,
                gl::RGB,
                gl::FLOAT,
                pixels.as_ptr() as *const c_void,
            );
        }
    }

    fn render(&self, _frame_ctx: &FrameRenderContext) {
        unsafe {
            gl::BindProgramPipeline(*self.pipeline);
            gl::BindVertexArray(*self.vao);
            gl::BindTextureUnit(0, *self.texture);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}
