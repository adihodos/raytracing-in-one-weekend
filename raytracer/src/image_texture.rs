use crate::{texture::Texture, types::Color};

pub struct ImageTexture {
    width: u32,
    height: u32,
    bytes_per_scanline: u32,
    pixels: Vec<u8>,
}

impl ImageTexture {
    pub fn blank() -> Self {
        Self {
            width: 0,
            height: 0,
            bytes_per_scanline: 0,
            pixels: Vec::new(),
        }
    }

    pub fn new<P: AsRef<std::path::Path>>(p: P) -> Self {
        use image::io::Reader as ImageReader;

        let img = ImageReader::open(p.as_ref())
            .expect(&format!(
                "Failed to open image file {}",
                p.as_ref().display()
            ))
            .decode()
            .expect(&format!("Failed to decode image {}", p.as_ref().display()))
            .into_rgba8();

        Self {
            width: img.width(),
            height: img.height(),
            bytes_per_scanline: img.width() * 4,
            pixels: img.to_vec(),
        }
    }

    pub fn from_pixels(width: u32, height: u32, pixels: &[u8]) -> Self {
        Self {
            width,
            height,
            bytes_per_scanline: (width * 4),
            pixels: pixels.to_vec(),
        }
    }
}

impl Texture for ImageTexture {
    fn value(
        &self,
        u: crate::types::Real,
        v: crate::types::Real,
        _point: crate::types::Point,
    ) -> crate::types::Color {
        if self.pixels.is_empty() {
            return (0f32, 1f32, 1f32).into();
        }

        //
        // Clamp input texture coordinates to [0,1] x [1,0]
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0); // Flip V to image coordinates

        let mut i = (u * self.width as f32) as i32;
        let mut j = (v * self.height as f32) as i32;

        //
        // Clamp integer mapping, since actual coordinates should be less than 1.0
        if i >= self.width as i32 {
            i = self.width as i32 - 1;
        };
        if j >= self.height as i32 {
            j = self.height as i32 - 1;
        }

        let color_scale = 1.0f32 / 255.0f32;
        let start_idx = j as usize * self.bytes_per_scanline as usize + i as usize * 4;

        Color::new(
            color_scale * self.pixels[start_idx + 0] as f32,
            color_scale * self.pixels[start_idx + 1] as f32,
            color_scale * self.pixels[start_idx + 2] as f32,
        )
    }
}
