use crate::{
    perlin::PerlinNoise,
    texture::Texture,
    types::{Color, Real},
};

pub struct NoiseTexture {
    perlin: PerlinNoise,
    scale: Real,
}

impl NoiseTexture {
    pub fn new(scale: Real) -> Self {
        Self {
            perlin: PerlinNoise::new(),
            scale,
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
        Color::broadcast(1f32)
            * 0.5f32
            * (1f32 + (self.scale * point.z + 10f32 * self.perlin.turbulence(point, 7)).sin())
    }
}
