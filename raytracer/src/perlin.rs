use rand::Rng;

use crate::types::{Point, Real};

pub struct PerlinNoise {
    randfloat: Vec<Real>,
    perm_x: Vec<i32>,
    perm_y: Vec<i32>,
    perm_z: Vec<i32>,
}

impl PerlinNoise {
    pub fn new() -> PerlinNoise {
        let mut rng = rand::thread_rng();

        // let mut v = vec![0f32; 256];
        // rng.fill(dest)

        PerlinNoise {
            randfloat: (0..256)
                .map(|_| rng.gen_range(0f32, 1f32))
                .collect::<Vec<_>>(),
            perm_x: gen_shuffled_vec(256, &mut rng),
            perm_y: gen_shuffled_vec(256, &mut rng),
            perm_z: gen_shuffled_vec(256, &mut rng),
        }
    }

    pub fn noise(&self, point: Point) -> Real {
        let i = ((4f32 * point.x) as usize) & 255;
        let j = ((4f32 * point.y) as usize) & 255;
        let k = ((4f32 * point.z) as usize) & 255;

        self.randfloat[(self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]) as usize]
    }
}

fn gen_shuffled_vec(elements: u32, rng: &mut rand::rngs::ThreadRng) -> Vec<i32> {
    let mut v = (0..elements).map(|i| i as i32).collect::<Vec<_>>();
    use rand::seq::SliceRandom;
    v.shuffle(rng);

    v
}
