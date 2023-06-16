use crate::types::{
    random_int, random_real, Real, Vec2, Vec3, C_HALF_ONE, C_ONE, C_TWO, C_TWO_PI, C_ZERO,
};
use num::{integer::Roots, Zero};

pub trait SampleStrategy {
    fn generate_samples(sets: u32, samples_in_set: u32) -> Vec<Vec2>;
}

#[derive(Clone)]
pub struct SamplerBase<T>
where
    T: SampleStrategy,
{
    samples: Vec<Vec2>,
    disk_samples: Vec<Vec2>,
    hemisphere_samples: Vec<Vec3>,
    shuffled_indices: Vec<u32>,
    sets: u32,
    samples_in_set: u32,
    count: usize,
    jump: usize,
    _a: std::marker::PhantomData<T>,
}

impl<T> SamplerBase<T>
where
    T: SampleStrategy,
{
    pub fn new(num_samples: i32, num_sets: Option<i32>) -> Self {
        let num_sets = num_sets.unwrap_or(83);
        let mut samples = T::generate_samples(num_sets as u32, num_samples as u32);

        //
        // shuffle x coords
        for p in 0..num_sets {
            for i in 0..num_samples - 1 {
                let target = random_int(0, num_samples - 1) + p * num_samples;
                let temp = samples[(i + p * num_samples + 1) as usize].x;
                samples[(i + p * num_samples + 1) as usize].x = samples[target as usize].x;
                samples[target as usize].x = temp;
            }
        }

        //
        // shuffle y coords
        for p in 0..num_sets {
            for i in 0..num_samples - 1 {
                let target = random_int(0, num_samples - 1) + p * num_samples;
                let temp = samples[(i + p * num_samples + 1) as usize].y;
                samples[(i + p * num_samples + 1) as usize].y = samples[target as usize].y;
                samples[target as usize].y = temp;
            }
        }

        let mut indices = (0..num_samples).map(|i| i as u32).collect::<Vec<_>>();
        let mut rng = rand::thread_rng();

        let mut shuffled_indices: Vec<u32> = Vec::new();
        (0..num_sets as u32).for_each(|_| {
            use rand::seq::SliceRandom;

            indices.shuffle(&mut rng);
            shuffled_indices.extend(indices.clone());
        });

        //
        // map unit square samples to unit disk
        let disk_samples = samples
            .iter()
            .map(|&s| {
                let sp = C_TWO * s - Vec2::broadcast(C_ONE);

                let (r, phi) = if sp.x > -sp.y {
                    if sp.x > sp.y {
                        // sector 1
                        (sp.x, sp.y / sp.x)
                    } else {
                        // sector2
                        (sp.y, C_TWO - sp.x / sp.y)
                    }
                } else {
                    if sp.x < sp.y {
                        // sector 3
                        (-sp.x, C_TWO * C_TWO + sp.y / sp.x)
                    } else {
                        // sector 4
                        let r = -sp.y;
                        let phi = if !sp.y.is_zero() {
                            6 as Real - sp.x / sp.y
                        } else {
                            C_ZERO
                        };

                        (r, phi)
                    }
                };

                let phi = phi * (0.25 as Real);
                let (sin_phi, cos_phi) = phi.sin_cos();

                Vec2 {
                    x: r * cos_phi,
                    y: r * sin_phi,
                }
            })
            .collect::<Vec<_>>();

        let hemisphere_samples = samples
            .iter()
            .map(|&s| {
                let (sin_phi, cos_phi) = (C_TWO_PI * s).x.sin_cos();
                let cos_theta = (C_ONE - s.y).powf(C_ONE / (C_ONE + std::f32::consts::E));
                let sin_theta = (C_ONE - cos_theta * cos_theta).sqrt();

                Vec3 {
                    x: sin_theta * cos_phi,
                    y: sin_theta * sin_phi,
                    z: cos_theta,
                }
            })
            .collect::<Vec<_>>();

        Self {
            samples,
            disk_samples,
            hemisphere_samples,
            shuffled_indices,
            count: 0,
            jump: 0,
            sets: num_sets as u32,
            samples_in_set: num_samples as u32,
            _a: std::marker::PhantomData::default(),
        }
    }

    pub fn sample_unit_square(&mut self) -> Vec2 {
        if self.count % self.samples_in_set as usize == 0 {
            self.jump = (random_int(0, std::i32::MAX - 1) % (self.sets as i32)
                * self.samples_in_set as i32) as usize;
        }

        let s = self.samples[self.jump
            + self.shuffled_indices[self.jump + (self.count % self.samples_in_set as usize)]
                as usize];

        self.count += 1;
        s
    }

    pub fn sample_unit_disk(&mut self) -> Vec2 {
        if self.count % self.samples_in_set as usize == 0 {
            self.jump = (random_int(0, std::i32::MAX - 1) % (self.sets as i32)
                * self.samples_in_set as i32) as usize;
        }

        let s = self.disk_samples[self.jump
            + self.shuffled_indices[self.jump + (self.count % self.samples_in_set as usize)]
                as usize];

        self.count += 1;
        s
    }

    pub fn sample_unit_hemisphere(&mut self) -> Vec3 {
        if self.count % self.samples_in_set as usize == 0 {
            self.jump = (random_int(0, std::i32::MAX - 1) % (self.sets as i32)
                * self.samples_in_set as i32) as usize;
        }

        let s = self.hemisphere_samples[self.jump
            + self.shuffled_indices[self.jump + (self.count % self.samples_in_set as usize)]
                as usize];

        self.count += 1;
        s
    }
}

#[derive(Clone)]
pub struct NRooksSamplingStrategy {}

impl SampleStrategy for NRooksSamplingStrategy {
    fn generate_samples(sets: u32, samples_in_set: u32) -> Vec<Vec2> {
        let mut samples = Vec::<Vec2>::with_capacity((sets * samples_in_set) as usize);

        for p in 0..sets {
            for j in 0..samples_in_set {
                let pt = Vec2 {
                    x: (j as Real + random_real()) / (samples_in_set as Real),
                    y: (j as Real + random_real()) / (samples_in_set as Real),
                };

                samples.push(pt);
            }
        }

        samples
    }
}

pub type NRooksSampler = SamplerBase<NRooksSamplingStrategy>;

#[derive(Clone)]
pub struct JitteredSamplingStrategy {}

impl SampleStrategy for JitteredSamplingStrategy {
    fn generate_samples(sets: u32, samples_in_set: u32) -> Vec<Vec2> {
        let mut samples = Vec::<Vec2>::with_capacity((sets * samples_in_set) as usize);
        let n = samples_in_set.sqrt();
        for _p in 0..sets {
            for j in 0..n {
                for k in 0..n {
                    let pt = Vec2 {
                        x: (k as Real + random_real()) / (n as Real),
                        y: (j as Real + random_real()) / (n as Real),
                    };

                    samples.push(pt);
                }
            }
        }

        samples
    }
}

pub type JitteredSampler = SamplerBase<JitteredSamplingStrategy>;

#[derive(Clone)]
pub struct MultiJitteredSamplingStrategy {}

impl SampleStrategy for MultiJitteredSamplingStrategy {
    fn generate_samples(sets: u32, samples_in_set: u32) -> Vec<Vec2> {
        let n = samples_in_set.sqrt();
        let subcell_width = (samples_in_set as Real).recip();
        let mut samples = Vec::<Vec2>::with_capacity((sets * samples_in_set) as usize);

        for _p in 0..sets {
            for i in 0..n {
                for j in 0..n {
                    let pt = Vec2 {
                        x: ((i * n + j) as Real) * subcell_width + random_real() * subcell_width,
                        y: ((j * n + i) as Real) * subcell_width + random_real() * subcell_width,
                    };

                    samples.push(pt);
                }
            }
        }
        samples
    }
}

pub type MultiJitteredSampler = SamplerBase<MultiJitteredSamplingStrategy>;

#[derive(Clone)]
pub struct SimpleSamplingStrategy {}

impl SampleStrategy for SimpleSamplingStrategy {
    fn generate_samples(sets: u32, samples_in_set: u32) -> Vec<Vec2> {
        let mut samples = Vec::<Vec2>::with_capacity((sets * samples_in_set) as usize);

        let n = samples_in_set.sqrt();
        for _j in 0..sets {
            for p in 0..n {
                for q in 0..n {
                    let pt = Vec2 {
                        x: (q as Real + C_HALF_ONE) / (n as Real),
                        y: (p as Real + C_HALF_ONE) / (n as Real),
                    };

                    samples.push(pt);
                }
            }
        }

        samples
    }
}

pub type SimpleSampler = SamplerBase<SimpleSamplingStrategy>;
