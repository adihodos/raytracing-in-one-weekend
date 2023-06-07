use crate::{texture::Texture, types::Color};

#[derive(Copy, Clone, Debug)]
pub struct SolidColorTexture {
    color: Color,
}

impl SolidColorTexture {
    pub fn new(color: impl std::convert::Into<Color>) -> Self {
        Self {
            color: color.into(),
        }
    }
}

impl Texture for SolidColorTexture {
    fn value(
        &self,
        _u: crate::types::Real,
        _v: crate::types::Real,
        _point: crate::types::Point,
    ) -> Color {
        self.color
    }
}
