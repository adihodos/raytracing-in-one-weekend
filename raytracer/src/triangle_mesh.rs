use std::sync::Arc;

use num::Zero;

use crate::{
    aabb3::Aabb,
    geometry_import::{self, GeometryNode, GeometryVertex, ImportedGeometry},
    hittable::{HitRecord, Hittable},
    image_texture::ImageTexture,
    lambertian::Lambertian,
    material::Material,
    types::{Mat4, Ray, Real, C_ONE, C_ZERO},
};

pub struct TriangleMesh {
    obj2world: Mat4,
    world2obj: Mat4,
    nodes: Vec<GeometryNode>,
    vertices: Vec<GeometryVertex>,
    aabb: Aabb,
    materials: Arc<Vec<Arc<dyn Material>>>,
    mtl: Arc<dyn Material>,
}

impl TriangleMesh {
    pub fn from_file<P: AsRef<std::path::Path>>(
        p: P,
        obj2world: Mat4,
        mtl: Arc<dyn Material>,
    ) -> TriangleMesh {
        let geometry = ImportedGeometry::import_from_file(&p)
            .expect(&format!("Failed to import mesh : {}", p.as_ref().display()));

        eprintln!(
            "Model: vertices {}, indices {}, nodes {}",
            geometry.vertices().len(),
            geometry.indices().len(),
            geometry.nodes().len()
        );

        Self::new(&geometry, obj2world, mtl)
    }
    pub fn new(
        imported_geometry: &ImportedGeometry,
        obj2world: Mat4,
        mtl: Arc<dyn Material>,
    ) -> Self {
        let world2obj = math::mat4::invert(&obj2world);
        let normals2world = world2obj.transpose();

        let mut aabb = Aabb::default();

        let vertices = imported_geometry
            .vertices()
            .iter()
            .map(|vtx| {
                //
                // transforming rays to object space is expensive for a mesh
                // so transform all position and normals to world space and recompute the AABB
                let pos = math::mat4::transform_point(&obj2world, vtx.pos);
                let normal =
                    math::vec3::normalize(math::mat4::transform_vector(&normals2world, vtx.normal));

                aabb.add_point(pos);

                GeometryVertex {
                    pos,
                    normal,
                    ..*vtx
                }
            })
            .collect::<Vec<_>>();

        let nodes = imported_geometry
            .nodes()
            .iter()
            .filter(|node| !node.indices.is_empty())
            .map(|node| {
                let aabb = node.indices.iter().fold(Aabb::default(), |mut bbox, &idx| {
                    bbox.add_point(vertices[idx as usize].pos);
                    bbox
                });

                GeometryNode {
                    aabb,
                    ..node.clone()
                }
            })
            .collect::<Vec<_>>();

        eprintln!("AABB {:?}", aabb);
        nodes.iter().for_each(|n| {
            eprintln!("node {} aabb {:?}", n.name, n.aabb);
        });

        let (img_width, img_height, copy_src) = imported_geometry.pbr_base_color_images();
        let materials = Arc::new(
            copy_src
                .iter()
                .map(|copy_img| {
                    let tex = ImageTexture::from_pixels(img_width, img_height, unsafe {
                        std::slice::from_raw_parts(copy_img.src, copy_img.bytes)
                    });

                    Arc::new(Lambertian::from_texture(Arc::new(tex))) as Arc<dyn Material>
                })
                .collect::<Vec<_>>(),
        );

        TriangleMesh {
            vertices,
            nodes,
            aabb,
            mtl,
            obj2world,
            world2obj,
            materials,
        }
    }

    fn ray_triangle_intersect(
        &self,
        idx: &[u32],
        r: &Ray,
        t_min: Real,
        t_max: Real,
    ) -> Option<HitRecord> {
        let p1 = &self.vertices[idx[0] as usize];
        let p2 = &self.vertices[idx[1] as usize];
        let p3 = &self.vertices[idx[2] as usize];

        use math::vec3::{cross, dot, normalize};

        let e1 = p2.pos - p1.pos;
        let e2 = p3.pos - p1.pos;

        let p = cross(r.direction, e2);
        let tmp = dot(p, e1);

        if tmp.is_zero() {
            return None;
        }

        let tmp = tmp.recip();
        let s = r.origin - p1.pos;

        let u = tmp * dot(s, p);
        if u < C_ZERO || u > C_ONE {
            return None;
        }

        let q = cross(s, e1);
        let v = tmp * dot(r.direction, q);

        if v < C_ZERO || (v + u) > C_ONE {
            return None;
        }

        let t = tmp * dot(e2, q);
        if t < t_min || t > t_max {
            return None;
        }

        // let uvs = [p1.uv, p2.uv, p3.uv];
        let w = C_ONE - u - v;

        let uv = u * p1.uv + v * p2.uv + w * p3.uv;

        // let tx_u = u * uvs[0].x + v * uvs[1].x + w * uvs[2].x;
        // let tx_v = u * uvs[0].y + v * uvs[1].y + w * uvs[2].y;

        let normal = normalize(u * p1.normal + v * p2.normal + w * p3.normal);

        //
        // cull back-faces
        if dot(r.direction, normal) > C_ZERO {
            return None;
        }

        let mtl = self.mtl.clone();
        //self.materials[p1.pbr_buf_id as usize].clone();
        Some(HitRecord::new(r.at(t), normal, r, t, mtl, uv.x, uv.y))
    }

    fn ray_triangle_intersect_test(
        &self,
        idx: &[u32],
        r: &Ray,
        t_min: Real,
        t_max: Real,
    ) -> Option<HitRecord> {
        //
        // Physically based rendering, section 3.6.2, pg 140
        let p1 = &self.vertices[idx[0] as usize];
        let p2 = &self.vertices[idx[1] as usize];
        let p3 = &self.vertices[idx[2] as usize];

        use math::vec3::{cross, dot, normalize};

        let e1 = p2.pos - p1.pos;
        let e2 = p3.pos - p1.pos;
        let s1 = cross(r.direction, e2);
        let div = dot(s1, e1);

        if div.is_zero() {
            return None;
        }

        let inv_div = div.recip();

        let d = r.origin - p1.pos;
        let b1 = dot(d, s1) * inv_div;

        if b1 < C_ZERO || b1 > C_ONE {
            return None;
        }

        let s2 = cross(d, e1);
        let b2 = dot(r.direction, s2) * inv_div;

        if b2 < C_ZERO || (b1 + b2) > C_ONE {
            return None;
        }

        let t = dot(e2, s2) * inv_div;
        if t < t_min || t > t_max {
            return None;
        }

        let b0 = C_ONE - b1 - b2;
        let n = normalize(b0 * p1.normal + b1 * p2.normal + b2 * p3.normal);
        if dot(r.direction, n) > C_ZERO {
            return None;
        }

        let uvs = b0 * p1.uv + b1 * p2.uv + b2 * p3.uv;

        // let mtl = self.materials[p1.pbr_buf_id as usize].clone();
        let mtl = self.mtl.clone();

        Some(HitRecord::new(r.at(t), n, r, t, mtl, uvs.x, uvs.y))
    }
}

impl Hittable for TriangleMesh {
    fn bounding_box(&self, _time0: Real, _time11: Real) -> Option<Aabb> {
        Some(self.aabb)
    }

    fn hit(&self, r: &Ray, t_min: Real, t_max: Real) -> Option<HitRecord> {
        if self.aabb.hit(r, t_min, t_max) {
            self.nodes
                .iter()
                .filter_map(|node| {
                    if node.aabb.hit(r, t_min, t_max) {
                        node.indices.chunks(3).find_map(|idx_range| {
                            self.ray_triangle_intersect_test(idx_range, r, t_min, t_max)
                        })
                    } else {
                        None
                    }
                })
                .reduce(|hit0, hit1| if hit0.t < hit1.t { hit0 } else { hit1 })
        } else {
            None
        }
    }
}
