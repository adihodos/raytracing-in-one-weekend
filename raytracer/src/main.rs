#![allow(dead_code)]

use std::{
    iter::FromIterator,
    os::raw::c_void,
    path::Path,
    sync::{mpsc::Receiver, Arc},
};

use checker_texture::CheckerTexture;
use diffuse_light::DiffuseLight;
use image_texture::ImageTexture;
use material::{Material, ScatterRecord};
use noise_texture::NoiseTexture;
use pdf::{HittablePdf, MixturePdf, Pdf};
use rectangles::XYRect;
use serde::{Deserialize, Serialize};

mod ui;

mod aabb3;
mod block;
mod bvh;
mod camera;
mod checker_texture;
mod constant_medium;
mod dielectric;
mod diffuse_light;
mod flip_face;
mod generic_handle;
mod geometry_import;
mod hittable;
mod hittable_list;
mod image_texture;
mod isotropic;
mod lambertian;
mod material;
mod metal;
mod noise_texture;
mod objects;
mod onb;
mod pdf;
mod perlin;
mod rectangles;
mod solid_color_texture;
mod texture;
mod transform;
mod types;

use dielectric::Dielectric;
use hittable::Hittable;
use hittable_list::HittableList;
use lambertian::Lambertian;
use metal::Metal;
use objects::sphere::Sphere;

use rand::{seq::SliceRandom, Rng};
use rendering::gl;
use types::*;

use glfw::Context;
use ui::UiBackend;

use crate::{
    block::Block,
    bvh::BvhNode,
    constant_medium::ConstantMedium,
    flip_face::FlipFace,
    objects::sphere::MovingSphere,
    rectangles::{XZRect, YZRect},
    transform::{RotateY, Translate},
};

#[derive(Copy, Clone)]
struct RaytracedPixel {
    x: u32,
    y: u32,
    color: Color,
}

const COLOR_CLAMP_MIN: Real = 0 as Real;
const COLOR_CLAMP_MAX: Real = 0.999 as Real;

fn ray_color(
    r: &Ray,
    background: Color,
    world: &HittableList,
    lights: Arc<dyn Hittable>,
    depth: i32,
) -> Color {
    if depth <= 0 {
        return Color::broadcast(0 as Real);
    }

    if let Some(rec) = world.hit(r, 0.001 as Real, C_INFINITY) {
        let emitted = rec.mtl.emitted(r, &rec, rec.u, rec.v, rec.p);
        if let Some(scatter) = rec.mtl.scatter(r, &rec) {
            return match scatter {
                ScatterRecord::SpecularRec { ray, attenuation } => {
                    attenuation * ray_color(&ray, background, world, lights, depth - 1)
                }
                ScatterRecord::PdfRec { pdf, attenuation } => {
                    let light_pdf = HittablePdf {
                        obj: lights.clone(),
                        origin: rec.p,
                    };

                    let mixed_pdf = MixturePdf::new(Arc::new(light_pdf), pdf);
                    let scattered_ray = Ray::new(rec.p, mixed_pdf.generate(), r.time);
                    let pdf_val = mixed_pdf.value(scattered_ray.direction);

                    emitted
                        + attenuation
                            * rec.mtl.scattering_pdf(r, &rec, &scattered_ray)
                            * ray_color(&scattered_ray, background, world, lights, depth - 1)
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

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
enum Scene {
    RandomWorld,
    TwoSpheres,
    TexturedSpheres,
    PerlinSpheres,
    SimpleLight,
    CornellBox,
    Chapter2Final,
    MeshTest,
}

fn scene_random_world() -> (HittableList, HittableList) {
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

    let light_mtl: Arc<DiffuseLight> = Arc::new((0f32, 0f32, 0f32).into());
    let mut lights = HittableList::new();
    lights.add(Arc::new(XZRect {
        x0: 213f32,
        x1: 343f32,
        z0: 227f32,
        z1: 332f32,
        k: 554f32,
        mtl: light_mtl.clone(),
    }));

    (world, lights)
}

fn scene_two_spheres() -> (HittableList, HittableList) {
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

    let light_mtl: Arc<DiffuseLight> = Arc::new((0f32, 0f32, 0f32).into());
    let mut lights = HittableList::new();
    lights.add(Arc::new(XZRect {
        x0: 213f32,
        x1: 343f32,
        z0: 227f32,
        z1: 332f32,
        k: 554f32,
        mtl: light_mtl.clone(),
    }));

    (world, lights)
}

fn scene_two_perlin_spheres() -> (HittableList, HittableList) {
    let noise_mtl = Arc::new(Lambertian::from_texture(Arc::new(NoiseTexture::new(3f32))));

    let mut world = HittableList::new();

    let grid_tex = Arc::new(Lambertian::from_texture(Arc::new(ImageTexture::new(
        "data/textures/uv_grids/ash_uvgrid01.jpg",
    ))));

    world.add(Arc::new(Sphere::new(
        Point::new(0f32, -1000f32, 0f32),
        1000f32,
        grid_tex,
    )));

    let grid_tex = Arc::new(Lambertian::from_texture(Arc::new(ImageTexture::new(
        "data/textures/uv_grids/ash_uvgrid03.jpg",
    ))));
    world.add(Arc::new(Sphere::new(
        Point::new(4f32, 4f32, 0f32),
        3f32,
        grid_tex,
    )));

    world.add(Arc::new(Sphere::new(
        Point::new(-6f32, 6f32, 2f32),
        6f32,
        noise_mtl.clone(),
    )));

    world.add(Arc::new(FlipFace {
        obj: Arc::new(XZRect {
            x0: -1000f32,
            x1: 1000f32,
            z0: -1000f32,
            z1: 1000f32,
            k: 1000f32,
            mtl: Arc::<DiffuseLight>::new((1f32, 1f32, 1f32).into()),
        }),
    }));

    let light_mtl: Arc<DiffuseLight> = Arc::new((0f32, 0f32, 0f32).into());
    let mut lights = HittableList::new();
    lights.add(Arc::new(XZRect {
        x0: 213f32,
        x1: 343f32,
        z0: 227f32,
        z1: 332f32,
        k: 554f32,
        mtl: light_mtl.clone(),
    }));

    (world, lights)
}

fn scene_textured_spheres() -> (HittableList, HittableList) {
    let image_texture = Arc::new(Lambertian::from_texture(Arc::new(ImageTexture::new(
        "data/textures/misc/earthmap.jpg",
    ))));

    let mut world = HittableList::new();

    world.add(Arc::new(Sphere::new(
        Point::new(0f32, 0f32, 0f32),
        2f32,
        image_texture.clone(),
    )));

    let light_mtl: Arc<DiffuseLight> = Arc::new((0f32, 0f32, 0f32).into());
    let mut lights = HittableList::new();
    lights.add(Arc::new(XZRect {
        x0: 213f32,
        x1: 343f32,
        z0: 227f32,
        z1: 332f32,
        k: 554f32,
        mtl: light_mtl.clone(),
    }));

    (world, lights)
}

fn scene_simple_light() -> (HittableList, HittableList) {
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

    let diffuse_light: Arc<DiffuseLight> = Arc::new((4f32, 4f32, 4f32).into());
    world.add(Arc::new(XYRect {
        x0: 3f32,
        x1: 5f32,
        y0: 1f32,
        y1: 3f32,
        k: -2f32,
        mtl: diffuse_light,
    }));

    let red_light: Arc<DiffuseLight> = Arc::new((4f32, 2f32, 0f32).into());
    world.add(Arc::new(Sphere::new(
        Point::new(0f32, 8f32, 0f32),
        2f32,
        red_light,
    )));

    let light_mtl: Arc<DiffuseLight> = Arc::new((0f32, 0f32, 0f32).into());
    let mut lights = HittableList::new();
    lights.add(Arc::new(XZRect {
        x0: 213f32,
        x1: 343f32,
        z0: 227f32,
        z1: 332f32,
        k: 554f32,
        mtl: light_mtl.clone(),
    }));

    (world, lights)
}

fn scene_cornell_box() -> (HittableList, HittableList) {
    let colors = [
        (0.65f32, 0.05f32, 0.05f32),
        (0.73f32, 0.73f32, 0.73f32),
        (0.12f32, 0.45f32, 0.15f32),
    ]
    .iter()
    .map(|color| Arc::new(Lambertian::new(*color)))
    .collect::<Vec<_>>();

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

    let light_mtl: Arc<DiffuseLight> = Arc::new((15f32, 15f32, 15f32).into());
    let light = Arc::new(FlipFace {
        obj: Arc::new(XZRect {
            x0: 213f32,
            x1: 343f32,
            z0: 227f32,
            z1: 332f32,
            k: 554f32,
            mtl: light_mtl,
        }),
    });
    world.add(light);

    let aluminium = Arc::new(Metal::new((0.8f32, 0.85f32, 0.88f32), 0f32));

    let box1 = Arc::new(Block::new(
        (0f32, 0f32, 0f32),
        (165f32, 330f32, 165f32),
        colors[1].clone(),
    ));
    let box1 = Arc::new(RotateY::new(box1, 15f32));
    let box1 = Arc::new(Translate {
        obj: box1,
        offset: (265f32, 0f32, 295f32).into(),
    });
    world.add(box1);

    let box2 = Arc::new(Block::new(
        (0f32, 0f32, 0f32),
        (165f32, 165f32, 165f32),
        colors[1].clone(),
    ));
    let box2 = Arc::new(RotateY::new(box2, -18f32));
    let box2 = Arc::new(Translate {
        obj: box2,
        offset: (130f32, 0f32, 65f32).into(),
    });
    // world.add(box2);

    let glass = Arc::new(Dielectric::new(1.5f32));
    let glass_sphere = Arc::new(Sphere::new((190f32, 90f32, 190f32).into(), 90f32, glass));
    world.add(glass_sphere);

    let light_mtl: Arc<DiffuseLight> = Arc::new((0f32, 0f32, 0f32).into());
    let mut lights = HittableList::new();
    lights.add(Arc::new(XZRect {
        x0: 213f32,
        x1: 343f32,
        z0: 227f32,
        z1: 332f32,
        k: 554f32,
        mtl: light_mtl.clone(),
    }));

    lights.add(Arc::new(Sphere::new(
        (190f32, 90f32, 190f32).into(),
        90f32,
        light_mtl.clone(),
    )));

    (world, lights)
}

fn scene_cornell_box_smoke() -> HittableList {
    let colors = [
        (0.65f32, 0.05f32, 0.05f32),
        (0.73f32, 0.73f32, 0.73f32),
        (0.12f32, 0.45f32, 0.15f32),
    ]
    .iter()
    .map(|color| Arc::new(Lambertian::new(*color)))
    .collect::<Vec<_>>();

    let light: Arc<DiffuseLight> = Arc::new((7f32, 7f32, 7f32).into());

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
        x0: 113f32,
        x1: 443f32,
        z0: 127f32,
        z1: 432f32,
        k: 554f32,
        mtl: light,
    }));

    let box1 = Arc::new(Block::new(
        (0f32, 0f32, 0f32),
        (165f32, 330f32, 165f32),
        colors[1].clone(),
    ));
    let box1 = Arc::new(RotateY::new(box1, 15f32));
    let box1 = Arc::new(Translate {
        obj: box1,
        offset: (265f32, 0f32, 295f32).into(),
    });

    world.add(Arc::new(ConstantMedium::from_colored_object(
        box1,
        (0f32, 0f32, 0f32),
        0.01f32,
    )));

    let box2 = Arc::new(Block::new(
        (0f32, 0f32, 0f32),
        (165f32, 165f32, 165f32),
        colors[1].clone(),
    ));
    let box2 = Arc::new(RotateY::new(box2, -18f32));
    let box2 = Arc::new(Translate {
        obj: box2,
        offset: (130f32, 0f32, 65f32).into(),
    });

    world.add(Arc::new(ConstantMedium::from_colored_object(
        box2,
        (1f32, 1f32, 1f32),
        0.01f32,
    )));

    world
}

fn scene_final_chapter2() -> (HittableList, HittableList) {
    let mut rng = rand::thread_rng();
    let mut world = HittableList::new();

    let white = Arc::new(Lambertian::new((0.73f32, 0.73f32, 0.73f32)));

    let ground = Arc::new(Lambertian::new((0.48_f32, 0.83_f32, 0.53_f32)));

    const NUM_BOXES: i32 = 20;

    let mut boxlist = Vec::<Arc<dyn Hittable>>::with_capacity((NUM_BOXES * NUM_BOXES) as usize);

    (0..NUM_BOXES).for_each(|i| {
        (0..NUM_BOXES).for_each(|j| {
            let w = 100_f32;
            let x0 = -1000_f32 + i as f32 * w;
            let z0 = -1000_f32 + j as f32 * w;
            let y0 = 0_f32;
            let x1 = x0 + w;
            let y1 = rng.gen_range(1f32, 101f32);
            let z1 = z0 + w;

            boxlist.push(Arc::new(Block::new(
                Vec3::new(x0, y0, z0),
                Vec3::new(x1, y1, z1),
                ground.clone(),
            )));
        });
    });

    world.add(BvhNode::new(boxlist.as_mut_slice(), 0_f32, 1_f32));

    let light = Arc::new(DiffuseLight::from((17f32, 17f32, 17f32)));

    world.add(Arc::new(FlipFace {
        obj: Arc::new(XZRect {
            x0: 123_f32,
            x1: 423_f32,
            z0: 147_f32,
            z1: 412_f32,
            k: 554_f32,
            mtl: light.clone(),
        }),
    }));

    let center = Vec3::new(400_f32, 400_f32, 200_f32);

    world.add(Arc::new(MovingSphere::new(
        center,
        center + Vec3::new(30_f32, 0_f32, 0_f32),
        0_f32,
        1_f32,
        50_f32,
        Arc::new(Lambertian::new((0.7_f32, 0.3_f32, 0.1_f32))),
    )));

    world.add(Arc::new(Sphere::new(
        Vec3::new(260_f32, 150_f32, 45_f32),
        50_f32,
        Arc::new(Dielectric::new(1.5_f32)),
    )));

    world.add(Arc::new(Sphere::new(
        Vec3::new(0_f32, 150_f32, 145_f32),
        50_f32,
        Arc::new(Metal::new((0.8_f32, 0.8_f32, 0.9_f32), 10_f32)),
    )));

    let boundary = Arc::new(Sphere::new(
        Vec3::new(360_f32, 150_f32, 145_f32),
        70_f32,
        Arc::new(Dielectric::new(1.5_f32)),
    ));
    world.add(boundary.clone());

    world.add(Arc::new(ConstantMedium::from_colored_object(
        boundary.clone(),
        (0.2_f32, 0.4_f32, 0.9_f32),
        0.2_f32,
    )));

    let boundary = Arc::new(Sphere::new(
        Vec3::broadcast(0_f32),
        5000_f32,
        Arc::new(Dielectric::new(1.5_f32)),
    ));
    world.add(Arc::new(ConstantMedium::from_colored_object(
        boundary.clone(),
        Vec3::broadcast(1_f32),
        0.0001_f32,
    )));

    let emat = Arc::new(Lambertian::from_texture(Arc::new(ImageTexture::new(
        "data/textures/misc/earthmap.jpg",
    ))));
    world.add(Arc::new(Sphere::new(
        (400f32, 200f32, 400f32).into(),
        100f32,
        emat,
    )));

    let pertex = Arc::new(NoiseTexture::new(0.1_f32));
    world.add(Arc::new(Sphere::new(
        Vec3::new(220_f32, 280_f32, 300_f32),
        80_f32,
        Arc::new(Lambertian::from_texture(pertex)),
    )));

    const NUM_SPHERES: i32 = 1000;

    let mut boxlist2 = (0..NUM_SPHERES)
        .map(|_| -> Arc<dyn Hittable> {
            let center = Vec3::new(
                165_f32 * random_real(),
                165_f32 * random_real(),
                165_f32 * random_real(),
            );

            Arc::new(Sphere::new(center, 10f32, white.clone()))
        })
        .collect::<Vec<Arc<dyn Hittable>>>();

    let node = BvhNode::new(boxlist2.as_mut_slice(), 0f32, 1f32);
    let node = Arc::new(RotateY::new(node, 15f32));
    let node = Arc::new(Translate {
        obj: node,
        offset: (-100f32, 270f32, 395f32).into(),
    });

    world.add(node);

    let light_mtl: Arc<DiffuseLight> = Arc::new((0f32, 0f32, 0f32).into());
    let mut lights = HittableList::new();
    lights.add(Arc::new(XZRect {
        x0: 213f32,
        x1: 343f32,
        z0: 227f32,
        z1: 332f32,
        k: 554f32,
        mtl: light_mtl.clone(),
    }));

    // lights.add(Arc::new(Sphere::new(
    //     (190f32, 90f32, 190f32).into(),
    //     90f32,
    //     light_mtl.clone(),
    // )));

    (world, lights)
}

struct Mesh {
    geometry: geometry_import::ImportedGeometry,
    mtl: Arc<dyn Material>,
}

impl Mesh {
    fn from_file<P: AsRef<Path>>(p: P) -> Mesh {
        let geometry = geometry_import::ImportedGeometry::import_from_file(&p)
            .expect("Failed to import teapot model");
        eprintln!(
            "Model: vertices {}, indices {}, nodes {}, bounding box {:?}",
            geometry.vertices().len(),
            geometry.indices().len(),
            geometry.nodes().len(),
            geometry.aabb
        );

        Mesh {
            geometry,
            mtl: Arc::new(Lambertian::new((0f32, 1f32, 1f32))),
        }
    }

    fn triangle_ray_intersect(
        v0: &geometry_import::GeometryVertex,
        v1: &geometry_import::GeometryVertex,
        v2: &geometry_import::GeometryVertex,
        ray: &Ray,
        t_min: Real,
        t_max: Real,
        mtl: Arc<dyn Material>,
    ) -> Option<hittable::HitRecord> {
        use math::vec3::{are_on_the_same_plane_side, cross, dot, normalize};

        let c0 = v1.pos - v0.pos;
        let c1 = v2.pos - v1.pos;
        let n = normalize(cross(c0, c1));

        //
        // check if the ray hits the triangle plane (use v0 as origin)
        let d = dot(n, v0.pos);

        const EPSILON: Real = 1.0E-5 as Real;
        let b_dot_n = dot(ray.direction, n);

        if b_dot_n.abs() < EPSILON {
            //
            // ray is parallel or contained in the triangle's plane
            return None;
        }

        //
        // compute point of intersection on the triangle's plane
        let a_dot_n = dot(ray.origin, n);
        let t = (d - a_dot_n) / b_dot_n;

        if !(t < t_max && t > t_min) {
            //
            // intersection point is behind the ray
            return None;
        }

        let p = ray.at(t);

        let vertices = [v0.pos, v1.pos, v2.pos];

        //
        // check if the point lies inside the triangle
        let containment_tests_failed = [(0, 1), (1, 2), (2, 0)].iter().any(|vertex_indices| {
            // direction vector along the edge
            let edge_vec = vertices[vertex_indices.1] - vertices[vertex_indices.0];
            // direction vector from the vertex to the intersection point with the ray
            let intersect_point_vec = p - vertices[vertex_indices.0];
            // orthogonal vector to the above two vectors
            let orthogonal_vec = cross(edge_vec, intersect_point_vec);

            !are_on_the_same_plane_side(orthogonal_vec, n)
        });

        if containment_tests_failed {
            //
            // point is on the plane defined by the triangle's vertices but
            // outside the triangle
            return None;
        }

        //
        // Point lies inside the triangle
        Some(hittable::HitRecord::new(
            p, n, ray, t, mtl, v0.uv.x, v0.uv.y,
        ))
    }
}

impl Hittable for Mesh {
    fn bounding_box(&self, time0: Real, time1: Real) -> Option<aabb3::Aabb> {
        Some(self.geometry.aabb)
    }

    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<hittable::HitRecord> {
        if self.geometry.aabb.hit(r, t_min, t_max) {
            for node in self.geometry.nodes().iter() {
                if !node.aabb.hit(r, t_min, t_max) {
                    continue;
                }

                let start = node.index_range.start;
                let end = node.index_range.end;

                assert!((end - start) % 3 == 0);

                let mut i = 0usize;

                while i < end / 3 {
                    let v0 = self.geometry.vertices()[self.geometry.indices()[i + 0] as usize];
                    let v1 = self.geometry.vertices()[self.geometry.indices()[i + 1] as usize];
                    let v2 = self.geometry.vertices()[self.geometry.indices()[i + 2] as usize];

                    let intersect_result = Self::triangle_ray_intersect(
                        &v0,
                        &v1,
                        &v2,
                        r,
                        t_min,
                        t_max,
                        self.mtl.clone(),
                    );

                    if intersect_result.is_some() {
                        return intersect_result;
                    }

                    i += 3;
                }
            }
        }

        None
    }
}

fn scene_mesh() -> (HittableList, HittableList) {
    // let geometry =
    //     geometry_import::ImportedGeometry::import_from_file(&"data/models/teapot/pyramid.glb")
    //         .expect("Failed to import teapot model");
    // eprintln!(
    //     "Model: vertices {}, indices {}, nodes {}",
    //     geometry.vertices().len(),
    //     geometry.indices().len(),
    //     geometry.nodes().len()
    // );

    // geometry
    //     .nodes()
    //     .iter()
    //     .filter(|node| !node.index_range.is_empty())
    //     .for_each(|node| {
    //         eprintln!("Node {:?}, bbox {:?}", node.index_range, node.aabb);
    //     });

    let mut world = HittableList::new();

    //
    // add floor
    let floor_mtl = Arc::new(Lambertian::from_texture(Arc::new(ImageTexture::new(
        "data/textures/uv_grids/ash_uvgrid01.jpg",
    ))));

    let floor = Arc::new(XZRect {
        x0: -1000f32,
        x1: 1000f32,
        z0: -1000f32,
        z1: 1000f32,
        k: 0f32,
        mtl: floor_mtl,
    });

    world.add(floor);

    world.add(Arc::new(FlipFace {
        obj: Arc::new(XZRect {
            x0: -1000f32,
            x1: 1000f32,
            z0: -1000f32,
            z1: 1000f32,
            k: 1000f32,
            mtl: Arc::new(DiffuseLight::from((1f32, 1f32, 1f32))),
        }),
    }));

    // world.add(Arc::new(XZRect {
    //     x0: 0f32,
    //     x1: 4f32,
    //     z0: 0f32,
    //     z1: 4f32,
    //     k: 4f32,
    //     mtl: light_mtl.clone(),
    // }));

    // world.add(Arc::new(XYRect {
    //     x0: 0f32,
    //     x1: 4f32,
    //     y0: 0f32,
    //     y1: 4f32,
    //     k: 2f32,
    //     mtl: light_mtl.clone(),
    // }));

    // world.add(Arc::new(Mesh::from_file("data/models/teapot/pyramid.glb")));

    let block_mtl = Arc::new(Lambertian::from_texture(Arc::new(ImageTexture::new(
        "data/textures/uv_grids/ash_uvgrid03.jpg",
    ))));
    let block = Arc::new(Block::unit_cube(block_mtl));
    // world.add(block.clone());

    use math::quat;
    use math::vec3;

    let r = quat::to_rotation_matrix(quat::Quat::axis_angle(45 as Real, vec3::consts::unit_y()));
    let t = Mat4::translate((35f32, 35f32, 0f32).into());
    let s = Mat4::uniform_scale(35f32);
    let transformed_block = Arc::new(crate::transform::Transform::new(t * r * s, block.clone()));
    world.add(transformed_block);

    let r = random_rotation_matrix();
    let s = 24f32;
    let t = Mat4::translate((-35f32, 35f32, -20f32).into());
    let transformed_block = Arc::new(crate::transform::Transform::new(t * r * s, block.clone()));
    world.add(transformed_block);

    let mut lights = HittableList::new();
    lights.add(Arc::new(XZRect {
        x0: -1000f32,
        x1: 1000f32,
        z0: -1000f32,
        z1: 1000f32,
        k: 1000f32,
        mtl: Arc::<DiffuseLight>::new((0f32, 0f32, 0f32).into()),
    }));

    (world, lights)
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

#[derive(serde::Serialize, serde::Deserialize)]
struct RaytracerConfig {
    active_scene: Scene,
    default_params: RaytracerParams,
    defined_scenes: Vec<(Scene, Option<RaytracerParams>)>,
}

struct RaytracerState {
    params: RaytracerParams,
    workers: Vec<std::thread::JoinHandle<()>>,
    workblocks_done: std::sync::Arc<std::sync::atomic::AtomicI32>,
    total_workblocks: u32,
    image_pixels: Vec<Color>,
    cancel_token: Arc<std::sync::atomic::AtomicBool>,
    timestamp: std::time::Instant,
    raytracing_time: std::time::Duration,
    rx: std::sync::mpsc::Receiver<RaytracedPixel>,
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
    fn load_config() -> RaytracerConfig {
        let f = std::fs::File::open("data/config/raytracer.config.ron")
            .expect("Failed to open config file");

        ron::de::from_reader(f).expect("Failed to decode config file")
    }

    fn new() -> RaytracerState {
        let tracer_cfg = Self::load_config();

        let (scene_type, params) = tracer_cfg
            .defined_scenes
            .iter()
            .find(|(scene_type, _)| *scene_type == tracer_cfg.active_scene)
            .map(|(scene_type, scene_params)| {
                (
                    scene_type,
                    scene_params.unwrap_or(tracer_cfg.default_params),
                )
            })
            .expect("Specified scene not found ...");

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
        let (world, lights) = match scene_type {
            Scene::RandomWorld => scene_random_world(),
            Scene::CornellBox => scene_cornell_box(),
            Scene::Chapter2Final => scene_final_chapter2(),
            Scene::SimpleLight => scene_simple_light(),
            Scene::MeshTest => scene_mesh(),
            Scene::PerlinSpheres => scene_two_perlin_spheres(),
            Scene::TwoSpheres => scene_two_spheres(),
            _ => todo!("Unimplemented"),
        };

        use std::sync::Mutex;
        let workblocks = Arc::new(Mutex::new(workblocks));

        let workblocks_done = Arc::new(std::sync::atomic::AtomicI32::new(0));
        let cancel_token = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let world = Arc::new(world);
        let lights = Arc::new(lights);

        let (tx, rx) = std::sync::mpsc::channel::<RaytracedPixel>();

        let workers = (0..params.workers)
            .map(|worker_idx| {
                let workblocks = Arc::clone(&workblocks);
                let world = Arc::clone(&world);

                let workblocks_done = Arc::clone(&workblocks_done);
                let cancel_token = Arc::clone(&cancel_token);
                let light = lights.clone();
                let tx = tx.clone();

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
                                                    light.clone(),
                                                    params.max_ray_depth,
                                                )
                                        },
                                    );

                                    let gamma_correct =
                                        1 as Real / params.samples_per_pixel as Real;

                                    let gamma_correct_fn = |x: Real| {
                                        (x * gamma_correct).sqrt().clamp(0 as Real, 1 as Real)
                                    };

                                    let check_invalid_pixel = |x: Real| !x.is_normal();

                                    let pixel_color = Vec3 {
                                        x: if check_invalid_pixel(pixel_color.x) {
                                            0 as Real
                                        } else {
                                            gamma_correct_fn(pixel_color.x)
                                        },
                                        y: if check_invalid_pixel(pixel_color.y) {
                                            0 as Real
                                        } else {
                                            gamma_correct_fn(pixel_color.y)
                                        },
                                        z: if check_invalid_pixel(pixel_color.z) {
                                            0 as Real
                                        } else {
                                            gamma_correct_fn(pixel_color.z)
                                        },
                                    };

                                    tx.send(RaytracedPixel {
                                        x: x as u32,
                                        y: y as u32,
                                        color: pixel_color,
                                    })
                                    .expect("Failed to send pixel to main");
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

        drop(tx);

        RaytracerState {
            total_workblocks,
            params,
            workers,
            workblocks_done,
            image_pixels: vec![
                Color::broadcast(0 as Real);
                (params.image_width * params.image_height) as usize
            ],
            cancel_token,
            timestamp: std::time::Instant::now(),
            raytracing_time: std::time::Duration::from_millis(0),
            rx,
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

    fn recv_pixels(&mut self) {
        while let Ok(pixel) = self.rx.try_recv() {
            self.image_pixels[(pixel.y * self.params.image_width as u32 + pixel.x) as usize] =
                pixel.color;
        }
    }
}

struct MainWindow {
    raytracer: RaytracerState,
    rtgl: RaytracingGlState,
    ui: UiBackend,
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    queue_screenshot: bool,
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
            queue_screenshot: false,
        }
    }

    fn main_loop(&mut self) {
        while !self.window.should_close() {
            self.glfw.poll_events();

            while let Ok((_, event)) = self.events.try_recv() {
                self.handle_window_event(event);
            }

            self.update_loop();

            if self.queue_screenshot {
                //
                // capture raytraced image
                let cont = unsafe {
                    std::slice::from_raw_parts(
                        self.raytracer.image_pixels.as_ptr() as *const _ as *const f32,
                        self.raytracer.image_pixels.len() * 3,
                    )
                };

                image::DynamicImage::ImageRgb32F(
                    image::ImageBuffer::from_raw(
                        (self.raytracer.params.image_width) as u32,
                        (self.raytracer.params.image_height) as u32,
                        cont.into(),
                    )
                    .expect("Failed to create image buffer"),
                )
                .to_rgb8()
                .save(format!(
                    "screenshots/raytraced_{}.png",
                    chrono::Local::now().format("%Y_%m_%d_%H_%M_%S")
                ))
                .expect("Failed to save image");

                //
                // capture framebuffer
                let (img_width, img_height) = self.window.get_framebuffer_size();
                let mut pixels = vec![0u8; (img_width * img_height * 3) as usize];

                unsafe {
                    gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
                    gl::NamedFramebufferReadBuffer(0, gl::BACK);
                    gl::ReadPixels(
                        0,
                        0,
                        img_width,
                        img_height,
                        gl::RGB,
                        gl::UNSIGNED_BYTE,
                        pixels.as_mut_ptr() as *mut c_void,
                    );
                }

                image::DynamicImage::ImageRgb8(
                    image::RgbImage::from_vec(img_width as u32, img_height as u32, pixels)
                        .expect("Failed to create image"),
                )
                .flipv()
                .save(format!(
                    "screenshots/framebuffer_{}.png",
                    chrono::Local::now().format("%Y_%m_%d_%H_%M_%S")
                ))
                .expect("Failed to save screenshot");

                self.queue_screenshot = false;
            }

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

            WindowEvent::Key(glfw::Key::F12, _, glfw::Action::Press, _) => {
                self.queue_screenshot = true;
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
        let mut queue_screenshot = self.queue_screenshot;

        ui.window("Status")
            .size([400f32, 600f32], imgui::Condition::FirstUseEver)
            .build(|| {
                let btn_color =
                    ui.push_style_color(imgui::StyleColor::Button, [0f32, 1f32, 0f32, 1f32]);
                let btn_color_active =
                    ui.push_style_color(imgui::StyleColor::ButtonActive, [1f32, 0f32, 0f32, 1f32]);

                if ui.button("Capture screenshot (F12)") {
                    queue_screenshot = true;
                }
                btn_color.pop();
                btn_color_active.pop();

                ui.separator();
                ui.text("---------- Image setup ----------");
                ui.text(format!("Image size: {}x{}", p.image_width, p.image_height));
                ui.text(format!("Aspect ratio: {}", p.aspect_ratio));

                ui.separator();
                ui.text("--------- Camera -----------");
                ui.text(format!("position: {}", Vec3::from(p.look_from)));
                ui.text(format!("look at: {}", Vec3::from(p.look_at)));
                ui.text(format!("world up: {}", Vec3::from(p.world_up)));
                ui.text(format!("Aperture: {}", p.aperture));
                ui.text(format!("Focus distance: {}", p.focus_dist));
                ui.text(format!("Field of view: {}", p.vertical_fov));

                ui.separator();
                ui.text("--------- Raytracer setup ---------");
                ui.text(format!("Maximum ray depth: {}", p.max_ray_depth));
                ui.text(format!("Samples per pixel: {}", p.samples_per_pixel));
                ui.text(format!("Worker threads: {}", p.workers));
                ui.text(format!(
                    "Workblock dimensions {0}x{0} pixels",
                    p.worker_block_pixels
                ));

                ui.text(format!("Randomized workloads: {}", p.shuffle_workblocks));

                ui.separator();
                ui.text("--------- Execution status ---------");
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

        self.queue_screenshot = queue_screenshot;
    }

    fn update_loop(&mut self) {
        self.raytracer.recv_pixels();

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
    const VS_PROGRAM: &'static str = include_str!("../../data/shaders/quad.vert");
    const FS_PROGRAM: &'static str = include_str!("../../data/shaders/quad.frag");

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
