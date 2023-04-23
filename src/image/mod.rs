use std::{num::NonZeroU32, ops::Deref};

use image::{ImageBuffer, Pixel};
use imgref::ImgVec;
use num_traits::ToPrimitive;
use palette::{rgb::channels::Argb, Packed};

use crate::{
    error::{Error, Result},
    BlitBuffer, Color, ToBlitBuffer,
};

impl<P, Container> ToBlitBuffer for ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]>,
{
    fn to_blit_buffer_with_mask_color<C>(&self, mask_color: C) -> Result<BlitBuffer>
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
            NonZeroU32::new(width).ok_or(Error::ZeroWidth)?,
            127,
        )
    }

    fn to_blit_buffer_with_alpha(&self, alpha_treshold: u8) -> Result<BlitBuffer> {
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
            NonZeroU32::new(width).ok_or(Error::ZeroWidth)?,
            alpha_treshold,
        )
    }
}

/// Convert separate u8 color components into a single packed color.
///
/// The type is `u64` because that's the base conversion type of the `num_traits` crate.
#[inline(always)]
fn color_from_u64(a: u64, r: u64, g: u64, b: u64) -> Color {
    ((a << 24) | (r << 16) | (g << 8) | b) as Color
}
