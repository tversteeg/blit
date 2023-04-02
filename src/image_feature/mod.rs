use std::ops::Deref;

use image::{ImageBuffer, Pixel};
use num_traits::NumCast;
use palette::{rgb::channels::Argb, Packed};

use crate::{BlitBuffer, BlitExt, Color};

impl<P, Container> BlitExt for ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]>,
{
    fn to_blit_buffer_with_mask_color<C>(&self, mask_color: C) -> BlitBuffer
    where
        C: Into<Packed<Argb>>,
    {
        let (width, _height) = self.dimensions();

        // Remove the alpha channel
        let mask_color = mask_color.into().color | 0xFF_00_00_00;

        BlitBuffer::from_iter(
            self.pixels().map(|pixel| pixel.to_rgba()).map(|pixel| {
                let pixel = color_from_u8(
                    0xFF,
                    NumCast::from(pixel[0]).unwrap(),
                    NumCast::from(pixel[1]).unwrap(),
                    NumCast::from(pixel[2]).unwrap(),
                );

                // If the pixel matches the mask color return nothing
                if pixel == mask_color {
                    0x00
                } else {
                    pixel
                }
            }),
            width as i32,
            127,
        )
    }

    fn to_blit_buffer_with_alpha(&self, alpha_treshold: u8) -> BlitBuffer {
        let (width, _height) = self.dimensions();

        BlitBuffer::from_iter(
            self.pixels().map(|pixel| pixel.to_rgba()).map(|pixel| {
                color_from_u8(
                    // RGBA -> ARGB
                    NumCast::from(pixel[3]).unwrap(),
                    NumCast::from(pixel[0]).unwrap(),
                    NumCast::from(pixel[1]).unwrap(),
                    NumCast::from(pixel[2]).unwrap(),
                )
            }),
            width as i32,
            alpha_treshold,
        )
    }
}

/// Convert separate u8 color components into a single packed color.
fn color_from_u8(a: u8, r: u8, g: u8, b: u8) -> Color {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}
