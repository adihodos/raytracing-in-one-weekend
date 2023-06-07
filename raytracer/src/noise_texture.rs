use crate::{perlin::PerlinNoise, texture::Texture, types::Color};

pub struct NoiseTexture {
    perlin: PerlinNoise,
}

impl NoiseTexture {
    pub fn new() -> Self {
        Self {
            perlin: PerlinNoise::new(),
        }
    }
}

impl Texture for NoiseTexture {
    fn value(
        &self,
        _u: crate::types::Real,
        _v: crate::types::Real,
        point: crate::types::Point,
    ) -> crate::types::Color {
        Color::broadcast(1f32) * self.perlin.noise(point)
    }
}
