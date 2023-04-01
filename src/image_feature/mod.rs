use super::*;
use image::*;
use num_traits::NumCast;

const RGBA_ALPHA_TRESHOLD: u8 = 127;

/// Create a `BlitBuffer` type from a generic image type so a `as_rgb*` step is not needed.
pub fn blit_buffer<I, P, S, C>(image: &I, mask_color: C) -> BlitBuffer
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
    C: Into<Color>,
{
    let mask_color = mask_color.into();
    let data = image
        .pixels()
        .map(|(_, _, p): (_, _, P)| {
            let pixel = p.to_rgba();

            // Convert pixel to Color
            let raw = Color::from_u8(
                NumCast::from(pixel[0]).unwrap(),
                NumCast::from(pixel[1]).unwrap(),
                NumCast::from(pixel[2]).unwrap(),
            );
            let raw_alpha: u8 = NumCast::from(pixel[3]).unwrap();

            if raw == mask_color || raw_alpha < RGBA_ALPHA_TRESHOLD {
                // Empty mask
                (Color::black(), Color::white())
            } else {
                // Filled with pixel value
                (raw, Color::alpha())
            }
        })
        .collect();

    BlitBuffer {
        width: image.width() as i32,
        height: image.height() as i32,
        data,
        mask_color,
    }
}

impl BlitExt for RgbImage {
    fn to_blit_buffer<C>(&self, mask_color: C) -> BlitBuffer
    where
        C: Into<Color>,
    {
        let mask_color = mask_color.into();

        let (width, height) = self.dimensions();

        let pixels = (width * height) as usize;
        let mut data: Vec<(Color, Color)> = vec![(Color::from_u32(0), Color::from_u32(0)); pixels];

        let mut index = 0;
        for y in 0..height {
            for x in 0..width {
                let pixel = self.get_pixel(x, y).0;

                // Convert pixel to Color
                let raw = Color::from_u8(pixel[0], pixel[1], pixel[2]);

                if raw == mask_color {
                    data[index].1 = Color::white();
                } else {
                    data[index].0 = raw;
                }

                index += 1;
            }
        }

        BlitBuffer {
            width: width as i32,
            height: height as i32,
            data,
            mask_color,
        }
    }

    fn blit<C>(&self, dst: &mut [u32], dst_width: usize, offset: (i32, i32), mask_color: C)
    where
        C: Into<Color>,
    {
        let mask_color = mask_color.into();

        let dst_size = (dst_width as i32, (dst.len() / dst_width) as i32);

        let (width, height) = self.dimensions();

        // Make sure only the pixels get rendered that are inside the dst
        let min_x = std::cmp::max(-offset.0, 0);
        let min_y = std::cmp::max(-offset.1, 0);

        let max_x = std::cmp::min(dst_size.0 - offset.0, width as i32);
        let max_y = std::cmp::min(dst_size.1 - offset.1, height as i32);

        for y in min_y..max_y {
            for x in min_x..max_x {
                let pixel = self.get_pixel(x as u32, y as u32).0;

                // Convert pixel to Color
                let raw = Color::from_u8(pixel[0], pixel[1], pixel[2]);

                // Check if the pixel isn't the mask
                if raw != mask_color {
                    // Apply the offsets
                    let dst_x = (x + offset.0) as usize;
                    let dst_y = (y + offset.1) as usize;

                    // Calculate the index
                    let index = dst_x + dst_y * dst_size.0 as usize;
                    dst[index] = raw.u32();
                }
            }
        }
    }
}

impl BlitExt for RgbaImage {
    fn to_blit_buffer<C>(&self, mask_color: C) -> BlitBuffer
    where
        C: Into<Color>,
    {
        let mask_color = mask_color.into();
        let data = self
            .pixels()
            .map(|p: &Rgba<u8>| {
                // Convert pixel to Color
                let raw = Color::from_u8(p[0], p[1], p[2]);
                if raw == mask_color || p[3] < RGBA_ALPHA_TRESHOLD {
                    // Empty mask
                    (Color::black(), Color::white())
                } else {
                    // Filled with pixel value
                    (raw, Color::alpha())
                }
            })
            .collect();

        BlitBuffer {
            width: self.width() as i32,
            height: self.height() as i32,
            data,
            mask_color,
        }
    }

    fn blit<C>(&self, dst: &mut [u32], dst_width: usize, offset: (i32, i32), mask_color: C)
    where
        C: Into<Color>,
    {
        let mask_color = mask_color.into();

        let dst_size = (dst_width as i32, (dst.len() / dst_width) as i32);

        let (width, height) = self.dimensions();

        // Make sure only the pixels get rendered that are inside the dst
        let min_x = std::cmp::max(-offset.0, 0);
        let min_y = std::cmp::max(-offset.1, 0);

        let max_x = std::cmp::min(dst_size.0 - offset.0, width as i32);
        let max_y = std::cmp::min(dst_size.1 - offset.1, height as i32);

        for y in min_y..max_y {
            for x in min_x..max_x {
                let pixel = self.get_pixel(x as u32, y as u32).0;

                // Convert pixel to Color
                let raw = Color::from_u8(pixel[0], pixel[1], pixel[2]);

                // Check if the pixel isn't the mask
                if raw != mask_color && pixel[3] >= RGBA_ALPHA_TRESHOLD {
                    // Apply the offsets
                    let dst_x = (x + offset.0) as usize;
                    let dst_y = (y + offset.1) as usize;

                    // Calculate the index
                    let index = dst_x + dst_y * dst_size.0 as usize;
                    dst[index] = raw.u32();
                }
            }
        }
    }
}
