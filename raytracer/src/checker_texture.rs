use crate::{solid_color_texture::SolidColorTexture, texture::Texture, types::Color};

#[derive(Clone)]
pub struct CheckerTexture {
    odd: std::sync::Arc<dyn Texture>,
    even: std::sync::Arc<dyn Texture>,
}

impl CheckerTexture {
    pub fn from_colors<C: std::convert::Into<Color>>(odd: C, even: C) -> Self {
        Self::new(
            std::sync::Arc::new(SolidColorTexture::new(odd.into())),
            std::sync::Arc::new(SolidColorTexture::new(even.into())),
        )
    }

    pub fn new(odd: std::sync::Arc<dyn Texture>, even: std::sync::Arc<dyn Texture>) -> Self {
        Self { odd, even }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: crate::types::Real, v: crate::types::Real, p: crate::types::Point) -> Color {
        let sines = (10f32 * p.x).sin() * (10f32 * p.y).sin() * (10f32 * p.z).sin();
        if sines < 0f32 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}
