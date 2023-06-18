use crate::{
    solid_color_texture::SolidColorTexture,
    texture::Texture,
    types::{Color, Real},
};

#[derive(Clone)]
pub struct CheckerTexture {
    odd: std::sync::Arc<dyn Texture>,
    even: std::sync::Arc<dyn Texture>,
    repeat_factor: Real,
}

impl CheckerTexture {
    pub fn from_colors<C: std::convert::Into<Color>>(odd: C, even: C, repeat_factor: Real) -> Self {
        Self::new(
            std::sync::Arc::new(SolidColorTexture::new(odd.into())),
            std::sync::Arc::new(SolidColorTexture::new(even.into())),
            repeat_factor,
        )
    }

    pub fn new(
        odd: std::sync::Arc<dyn Texture>,
        even: std::sync::Arc<dyn Texture>,
        repeat_factor: Real,
    ) -> Self {
        Self {
            odd,
            even,
            repeat_factor,
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: crate::types::Real, v: crate::types::Real, p: crate::types::Point) -> Color {
        let sines = (self.repeat_factor * p.x).sin()
            * (self.repeat_factor * p.y).sin()
            * (self.repeat_factor * p.z).sin();
        if sines < 0f32 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}
