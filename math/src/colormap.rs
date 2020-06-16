use super::colors::RGBAColor;
use super::utility::saturate;
use super::color_palette::basic;


pub fn create_linear_colormap(start: RGBAColor, end: RGBAColor, num: u32) -> Vec<RGBAColor> {
    let r0 = start.r as f32 / 255f32;
    let g0 = start.g as f32 / 255f32;
    let b0 = start.b as f32 / 255f32;

    if num > 1 {
        let r_inc = (end.r as f32 / 255f32 - r0) as f32 / (num as f32 - 1f32);
        let g_inc = (end.g as f32 / 255f32 - g0) as f32 / (num as f32 - 1f32);
        let b_inc = (end.b as f32 / 255f32 - b0) as f32 / (num as f32 - 1f32);

        (0..num)
            .map(|i| {
                RGBAColor::new(
                    (saturate(r0 + (i as f32 * r_inc)) * 255f32) as u8,
                    (saturate(g0 + (i as f32 * g_inc)) * 255f32) as u8,
                    (saturate(b0 + (i as f32 * b_inc)) * 255f32) as u8,
                )
            })
            .collect::<Vec<_>>()
    } else {
        vec![start]
    }
}
pub struct ColorMap {}

impl ColorMap {
    pub fn create_linear(start: RGBAColor, end: RGBAColor, num: u32) -> Vec<RGBAColor> {
        let r0 = start.r as f32 / 255f32;
        let g0 = start.g as f32 / 255f32;
        let b0 = start.b as f32 / 255f32;
        if num > 1 {
            let r_inc = (end.r as f32 / 255f32 - r0) as f32 / (num as f32 - 1f32);
            let g_inc = (end.g as f32 / 255f32 - g0) as f32 / (num as f32 - 1f32);
            let b_inc = (end.b as f32 / 255f32 - b0) as f32 / (num as f32 - 1f32);
            (0..num)
                .map(|i| {
                    RGBAColor::new(
                        (saturate(r0 + (i as f32 * r_inc)) * 255f32) as u8,
                        (saturate(g0 + (i as f32 * g_inc)) * 255f32) as u8,
                        (saturate(b0 + (i as f32 * b_inc)) * 255f32) as u8,
                    )
                })
                .collect::<Vec<_>>()
        } else {
            vec![start]
        }
    }

    /// Creates a palette : Dark Blue -> Cyan -> Green
    pub fn pf1() -> Vec<RGBAColor> {
        std::iter::once(RGBAColor::from(basic::BLACK))
            .chain(
                ColorMap::create_linear(
                    RGBAColor::from(basic::DK_BLUE),
                    RGBAColor::from(basic::CYAN),
                    128,
                )
                .into_iter()
                .take(127),
            )
            .chain(
                ColorMap::create_linear(
                    RGBAColor::from(basic::CYAN),
                    RGBAColor::from(basic::DK_GREEN),
                    128,
                )
                .into_iter(),
            )
            .collect()
    }

    /// Creates a palette: Dark Red -> Yellow -> Blue
    pub fn pf2() -> Vec<RGBAColor> {
        std::iter::once(RGBAColor::from(basic::BLACK))
            .chain(
                ColorMap::create_linear(
                    RGBAColor::from(basic::DK_RED),
                    RGBAColor::from(basic::YELLOW),
                    128,
                )
                .into_iter()
                .take(127),
            )
            .chain(
                ColorMap::create_linear(
                    RGBAColor::from(basic::DK_BLUE),
                    RGBAColor::from(basic::CYAN),
                    128,
                )
                .into_iter(),
            )
            .collect()
    }

    /// Drak blue -> cyan for the star drawing algo.
    pub fn pf3() -> Vec<RGBAColor> {
        std::iter::once(RGBAColor::from(basic::BLACK))
            .chain(
                ColorMap::create_linear(
                    RGBAColor::from(basic::CYAN),
                    RGBAColor::from(basic::DK_BLUE),
                    255,
                )
                .into_iter(),
            )
            .collect()
    }

    /// Increments red and green in 15 steps, used by the bands coloring scheme.
    pub fn pf4() -> Vec<RGBAColor> {
        let mut palette = vec![RGBAColor::from(basic::BLACK); 256];

        (0..15).for_each(|i| {
            let green = 255 - 16 * i;

            (0..15).for_each(|j| {
                let red = 255 - 12 * j;
                let idx = 16 * i + j + 17;
                palette[idx as usize] = RGBAColor::new(red as u8, green as u8, 0);
            });
        });

        palette
    }

    /// Increments blue and green in 15 steps, used by the bands coloring scheme.
    pub fn pf5() -> Vec<RGBAColor> {
        let mut palette = vec![RGBAColor::from(basic::BLACK); 256];

        (0..15).for_each(|i| {
            let green = 255 - 16 * i;

            (0..15).for_each(|j| {
                let blue = 255 - 12 * j;
                let idx = 16 * i + j + 17;
                palette[idx as usize] = RGBAColor::new(0, green as u8, blue as u8);
            });
        });

        palette
    }

    /// 4 colors for the quadrants scheme, 256 colors total
    pub fn pf6() -> Vec<RGBAColor> {
        let mut palette = vec![RGBAColor::from(basic::BLACK); 256];

        (1u32..129u32).for_each(|i| {
            palette[i as usize] = RGBAColor::new(0, 255.min(2 * i) as u8, 255.min(128 + i) as u8);
        });

        (1..128).for_each(|i| {
            palette[(128 + i) as usize] =
                RGBAColor::new((2 * i + 1) as u8, (255 - 2 * i) as u8, (255 - 2 * i) as u8);
        });

        palette
    }
    /// Yellow and blue bands, 256, colors
    pub fn pf7() -> Vec<RGBAColor> {
        let yellow_blue = [
            RGBAColor::from(basic::YELLOW),
            RGBAColor::from(basic::DK_BLUE),
        ];
        std::iter::once(RGBAColor::from(basic::BLACK))
            .chain((1..256).map(|i| yellow_blue[i % 2]))
            .collect()
    }

    /// Thin red bands, 256 colors
    pub fn pf8() -> Vec<RGBAColor> {
        let mut palette = vec![RGBAColor::from(basic::WHITE); 256];
        palette[0] = RGBAColor::from(basic::BLACK);

        (0..4).for_each(|i| {
            (0..3).for_each(|j| {
                palette[64 * i + j + 1] = RGBAColor::new(128, 0, 0);
                palette[64 * i + j + 4] = RGBAColor::new(192, 0, 0);
                palette[64 * i + j + 7] = RGBAColor::new(255, 0, 0);
                palette[64 * i + j + 10] = RGBAColor::new(255, 64, 64);
                palette[64 * i + j + 13] = RGBAColor::new(255, 128, 128);
                palette[64 * i + j + 16] = RGBAColor::new(255, 192, 192);
                palette[64 * i + j + 33] = RGBAColor::new(128, 64, 0);
                palette[64 * i + j + 36] = RGBAColor::new(192, 96, 0);
                palette[64 * i + j + 39] = RGBAColor::new(255, 128, 0);
                palette[64 * i + j + 42] = RGBAColor::new(255, 160, 64);
                palette[64 * i + j + 44] = RGBAColor::new(255, 192, 128);
                palette[64 * i + j + 47] = RGBAColor::new(255, 224, 192);
            });
        });

        palette
    }
}
