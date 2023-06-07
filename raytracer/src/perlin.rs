use rand::{thread_rng, Rng};

use crate::types::Vec3;

pub struct PerlinNoise {
    randfloat: Vec<Vec3>,
    perm_x: Vec<i32>,
    perm_y: Vec<i32>,
    perm_z: Vec<i32>,
}

fn gen_shuffled_vec(elements: u32) -> Vec<i32> {
    let mut v = (0..elements).map(|i| i as i32).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();

    use rand::seq::SliceRandom;
    v.shuffle(&mut rng);

    v
}

fn trilinear_interp(c: &[[[f32; 2]; 2]], u: f32, v: f32, w: f32) -> f32 {
    let mut accum = 0_f32;
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                accum += (i as f32 * u + (1 - i) as f32 * (1_f32 - u))
                    * (j as f32 * v + (1 - j) as f32 * (1_f32 - v))
                    * (k as f32 * w + (1 - k) as f32 * (1_f32 - w))
                    * c[i as usize][j as usize][k as usize];
            }
        }
    }

    accum
}

fn perlin_interp(c: &[[[Vec3; 2]; 2]], u: f32, v: f32, w: f32) -> f32 {
    let uu = smooth(u);
    let vv = smooth(v);
    let ww = smooth(w);

    let mut accum = 0_f32;

    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let weight = Vec3::new(u - i as f32, v - j as f32, w - k as f32);
                accum += (i as f32 * uu + (1_f32 - i as f32) * (1_f32 - uu))
                    * (j as f32 * vv + (1_f32 - j as f32) * (1_f32 - vv))
                    * (k as f32 * ww + (1_f32 - k as f32) * (1_f32 - ww))
                    * math::vec3::dot(c[i as usize][j as usize][k as usize], weight);
            }
        }
    }

    accum
}

fn smooth(t: f32) -> f32 {
    t * t * (3_f32 - 2_f32 * t)
}

impl PerlinNoise {
    pub fn new() -> PerlinNoise {
        let mut rng = thread_rng();

        PerlinNoise {
            randfloat: (0..256)
                .map(|_| {
                    math::vec3::normalize(Vec3::new(
                        -1_f32 + 2_f32 * rng.gen::<f32>(),
                        -1_f32 + 2_f32 * rng.gen::<f32>(),
                        -1_f32 + 2_f32 * rng.gen::<f32>(),
                    ))
                })
                .collect::<Vec<_>>(),
            perm_x: gen_shuffled_vec(256),
            perm_y: gen_shuffled_vec(256),
            perm_z: gen_shuffled_vec(256),
        }
    }

    pub fn noise(&self, p: Vec3) -> f32 {
        let u = smooth(p.x - p.x.floor());
        let v = smooth(p.y - p.y.floor());
        let w = smooth(p.z - p.z.floor());

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;

        let mut c: [[[Vec3; 2]; 2]; 2] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let idx = self.perm_x[((i + di) & 255) as usize]
                        ^ self.perm_y[((j + dj) & 255) as usize]
                        ^ self.perm_z[((k + dk) & 255) as usize];

                    c[di as usize][dj as usize][dk as usize] = self.randfloat[idx as usize];
                }
            }
        }

        perlin_interp(&c, u, v, w)
    }

    pub fn turbulence(&self, p: Vec3, depth: i32) -> f32 {
        let mut temp_p = p;
        let mut weight = 1_f32;

        (0..depth)
            .fold(0_f32, |acc, _| {
                let acc = acc + weight * self.noise(temp_p);
                weight *= 0.5_f32;
                temp_p *= 2_f32;

                acc
            })
            .abs()
    }
}
