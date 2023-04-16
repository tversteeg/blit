use std::ops::Deref;

use image::{ImageBuffer, Pixel};
use imgref::ImgVec;
use num_traits::ToPrimitive;
use palette::{rgb::channels::Argb, Packed};

use crate::{BlitBuffer, Color, ToBlitBuffer};

impl<P, Container> ToBlitBuffer for ImageBuffer<P, Container>
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
                let pixel = color_from_u64(
                    0xFF,
                    ToPrimitive::to_u64(&pixel[0]).unwrap_or(0x0),
                    ToPrimitive::to_u64(&pixel[1]).unwrap_or(0x0),
                    ToPrimitive::to_u64(&pixel[2]).unwrap_or(0x0),
                );

                // If the pixel matches the mask color return nothing
                if pixel == mask_color {
                    0x0
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
                color_from_u64(
                    // RGBA -> ARGB
                    ToPrimitive::to_u64(&pixel[3]).unwrap_or(0x0),
                    ToPrimitive::to_u64(&pixel[0]).unwrap_or(0x0),
                    ToPrimitive::to_u64(&pixel[1]).unwrap_or(0x0),
                    ToPrimitive::to_u64(&pixel[2]).unwrap_or(0x0),
                )
            }),
            width as i32,
            alpha_treshold,
        )
    }

    fn to_img_with_mask_color<C>(&self, mask_color: C) -> ImgVec<u32>
    where
        C: Into<Packed<Argb>>,
    {
        let buf = self.to_blit_buffer_with_mask_color(mask_color);

        ImgVec::new(buf.data, buf.width as usize, buf.height as usize)
    }
}

/// Convert separate u8 color components into a single packed color.
///
/// The type is `u64` because that's the base conversion type of the `num_traits` crate.
#[inline(always)]
fn color_from_u64(a: u64, r: u64, g: u64, b: u64) -> Color {
    ((a << 24) | (r << 16) | (g << 8) | b) as Color
}
