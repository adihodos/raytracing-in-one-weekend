use crate::types::{random_int, random_real, Real, Vec2, C_HALF_ONE};
use num::integer::Roots;

pub trait SampleGen {
    fn generate_samples(sets: u32, samples_in_set: u32) -> Vec<Vec2>;
}

#[derive(Clone)]
pub struct SamplerBase<T>
where
    T: SampleGen,
{
    samples: Vec<Vec2>,
    shuffled_indices: Vec<u32>,
    sets: u32,
    samples_in_set: u32,
    count: usize,
    jump: usize,
    _a: std::marker::PhantomData<T>,
}

impl<T> SamplerBase<T>
where
    T: SampleGen,
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

        Self {
            samples,
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
}

#[derive(Clone)]
pub struct NRooksSamplingStrategy {}

impl SampleGen for NRooksSamplingStrategy {
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

impl SampleGen for JitteredSamplingStrategy {
    fn generate_samples(sets: u32, samples_in_set: u32) -> Vec<Vec2> {
        let mut samples = Vec::<Vec2>::with_capacity((sets * samples_in_set) as usize);
        let n = samples_in_set.sqrt();
        for p in 0..sets {
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

impl SampleGen for MultiJitteredSamplingStrategy {
    fn generate_samples(sets: u32, samples_in_set: u32) -> Vec<Vec2> {
        let n = samples_in_set.sqrt();
        let subcell_width = (samples_in_set as Real).recip();
        let mut samples = Vec::<Vec2>::with_capacity((sets * samples_in_set) as usize);

        for p in 0..sets {
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

impl SampleGen for SimpleSamplingStrategy {
    fn generate_samples(sets: u32, samples_in_set: u32) -> Vec<Vec2> {
        let mut samples = Vec::<Vec2>::with_capacity((sets * samples_in_set) as usize);

        let n = samples_in_set.sqrt();
        for j in 0..sets {
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
