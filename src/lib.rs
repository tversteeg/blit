//! Draw sprites quickly using a masking color or an alpha treshold.
//!
//! All colors can be constructed from the [`palette`](https://crates.io/crates/palette) crate or directly with an RGB `u32` where the alpha channel is ignored.
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "image")] mod test {
//! use blit::BlitExt;
//!
//! const WIDTH: usize = 180;
//! const HEIGHT: usize = 180;
//! const MASK_COLOR: u32 = 0xFF_00_FF;
//! # fn main() {
//! // Create a buffer in which we'll draw our image
//! let mut buffer: Vec<u32> = vec![0xFF_FF_FF_FF; WIDTH * HEIGHT];
//!
//! // Load the image from disk using the `image` crate
//! let img = image::open("examples/smiley_rgb.png").unwrap().into_rgb8();
//!
//! // Blit by creating a special blitting buffer first where the MASK_COLOR will be the color that will be made transparent
//! let blit_buffer = img.to_blit_buffer_with_mask_color(MASK_COLOR);
//!
//! // Draw the image 2 times to the buffer
//! let pos = (10, 10);
//! blit_buffer.blit(&mut buffer, WIDTH, pos);
//! let pos = (20, 20);
//! blit_buffer.blit(&mut buffer, WIDTH, pos);
//! # }}
//! ```

#[cfg(feature = "aseprite")]
pub mod aseprite;
pub mod error;
#[cfg(feature = "image")]
pub mod image;

use std::ops::Range;

use palette::{rgb::channels::Argb, Packed};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Internal representation of a color.
type Color = u32;

/// A trait adding blitting functions to image types.
pub trait BlitExt {
    /// Convert the image to a custom `BlitBuffer` type which is optimized for applying the blitting operations.
    ///
    /// It's assumed that the alpha channel in the resulting pixel is properly set.
    /// The alpha treshold is the offset point at which an alpha value will be used as either a transparent pixel or a colored one.
    fn to_blit_buffer_with_alpha(&self, alpha_treshold: u8) -> BlitBuffer;

    /// Convert the image to a custom `BlitBuffer` type which is optimized for applying the blitting operations.
    ///
    /// Ignore the alpha channel if set and use only a single color for transparency.
    fn to_blit_buffer_with_mask_color<C>(&self, mask_color: C) -> BlitBuffer
    where
        C: Into<Packed<Argb>>;
}

/// A data structure holding a color and a mask buffer to make blitting on a buffer real fast.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct BlitBuffer {
    /// Image width in pixels.
    width: i32,
    /// Image height in pixels.
    height: i32,

    /// Vector of colors, the highest 8 bits are alpha and the remaining 24 bits the RGB color channels.
    data: Vec<Color>,
}

impl BlitBuffer {
    /// Create a instance from a buffer of ARGB data packed in a single `u32`.
    ///
    /// It's assumed that the alpha channel in the resulting pixel is properly set.
    /// The alpha treshold is the offset point at which an alpha value will be used as either a transparent pixel or a colored one.
    pub fn from_buffer(src: &[Color], width: i32, alpha_treshold: u8) -> Self {
        Self::from_iter(src.iter().copied(), width, alpha_treshold)
    }

    /// Create a instance from a iterator of ARGB data packed in a single `u32`.
    ///
    /// It's assumed that the alpha channel in the resulting pixel is properly set.
    /// The alpha treshold is the offset point at which an alpha value will be used as either a transparent pixel or a colored one.
    pub fn from_iter<I>(iter: I, width: i32, alpha_treshold: u8) -> Self
    where
        I: Iterator<Item = Color>,
    {
        // Shift the alpha to the highest bits so we can do a direct comparison without needing to shift every pixel again
        let alpha_treshold = (alpha_treshold as Color) << 24;

        // Create the data buffer filled with transparent pixels
        let data = iter
            .map(|pixel| {
                if pixel < alpha_treshold {
                    0x00_00_00_00
                } else {
                    pixel | 0xFF_00_00_00
                }
            })
            .collect::<Vec<_>>();

        // We can calculate the height from the total buffer
        let height = data.len() as i32 / width;

        Self {
            width,
            height,
            data,
        }
    }

    /// Blit the image on a buffer using bitwise operations.
    pub fn blit(&self, dst: &mut [u32], dst_width: usize, (offset_x, offset_y): (i32, i32)) {
        let dst_height = dst.len() / dst_width;

        let dst_width_i32 = dst_width as i32;
        let dst_height_i32 = dst_height as i32;

        // If the sizes match and the buffers are aligned we don't have to do any special bounds checks
        if (offset_x, offset_y) == (0, 0)
            && (dst_width_i32, dst_height_i32) == (self.width, self.height)
        {
            // Due to there being no bound overlap since the dimensions are exactly the same we can treat this case like a single contiguous horizontal strip
            self.blit_horizontal(dst, 0..dst.len(), 0..self.data.len());

            return;
        }

        // We can blit the sub rectangle but it's our entire size
        self.blit_rect(
            dst,
            dst_width,
            (offset_x, offset_y),
            (0, 0, self.width, self.height),
        )
    }

    /// Blit a section of the image on a buffer.
    ///
    /// The sub rectangle is the section of the source that will be rendered.
    /// Its values are (X, Y, Width, Height) in pixels.
    /// A good mental model to keep for the section is that it's a view on the blit buffer that will be rendered.
    pub fn blit_rect(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        (offset_x, offset_y): (i32, i32),
        (sub_rect_x, sub_rect_y, sub_rect_width, sub_rect_height): (i32, i32, i32, i32),
    ) {
        let dst_height = dst.len() / dst_width;

        let dst_width_i32 = dst_width as i32;
        let dst_height_i32 = dst_height as i32;

        // Region we need to draw in
        let dst_start_x = offset_x.max(0);
        let dst_start_y = offset_y.max(0);
        let dst_end_x = (offset_x + sub_rect_width).min(dst_width_i32) as usize;
        let dst_end_y = (offset_y + sub_rect_height).min(dst_height_i32) as usize;
        let dst_start_x_usize = dst_start_x as usize;
        let dst_start_y_usize = dst_start_y as usize;

        // Pixel ranges in the blit buffer that we need to dst
        let blit_pixel_start_x = (dst_start_x - offset_x + sub_rect_x) as usize;
        let blit_pixel_start_y = (dst_start_y - offset_y + sub_rect_y) as usize;

        // How many pixels we need to blit in total
        let pixels_x = dst_end_x - dst_start_x_usize;
        let pixels_y = dst_end_y - dst_start_y_usize;

        let blit_width_usize = self.width as usize;

        for y in 0..pixels_y {
            // Range of horizontal pixel we need to blit for this y
            let blit_start_x = blit_pixel_start_x + (blit_pixel_start_y + y) * blit_width_usize;
            let blit_end_x = blit_start_x + pixels_x;
            let dst_start_x = dst_start_x_usize + (dst_start_y_usize + y) * dst_width;
            let dst_end_x = dst_start_x + pixels_x;

            // Blit the horizontal slice
            self.blit_horizontal(dst, dst_start_x..dst_end_x, blit_start_x..blit_end_x);
        }
    }

    /// Get the size of the buffer in pixels.
    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// Get the width of the buffer in pixels.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Get the height of the buffer in pixels.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Blit a horizontal strip.
    fn blit_horizontal(&self, dst: &mut [u32], dst_index: Range<usize>, blit_index: Range<usize>) {
        // Same size iterators over both our buffer and the output buffer
        let blit_iter = self.data[blit_index].iter();
        let dst_iter = dst[dst_index].iter_mut();

        // Blit each pixel
        dst_iter.zip(blit_iter).for_each(|(dst_pixel, blit_pixel)| {
            *dst_pixel = Self::blit_pixel(*dst_pixel, *blit_pixel);
        });
    }

    /// Blit a single pixel.
    ///
    /// The main logic of calculating the resulting color that needs to be drawn.
    #[inline(always)]
    fn blit_pixel(dst_pixel: Color, blit_pixel: Color) -> Color {
        // Set the pixel from the blit image if the mask value is set
        if (blit_pixel >> 24) > 0 {
            // Pixel from the blit buffer is not masked, use it
            blit_pixel
        } else {
            // Pixel from the blit buffer is maskde, use the original color
            dst_pixel
        }
    }
}

impl std::fmt::Debug for BlitBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlitBuffer")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_fit() {
        let mut buffer = [0xFF, 0xFF_00, 0xFF_00_00, 0xFF, 0xFF_00, 0xFF_00_00];

        // The last number should be masked
        let blit = BlitBuffer::from_buffer(
            &[
                0xFF_00_00_AA,
                0xFF_00_AA_00,
                0xFF_AA_00_00,
                0xBB,
                0xBB,
                0xBB,
            ],
            2,
            127,
        );
        blit.blit(&mut buffer, 2, (0, 0));

        // Create a copy but cast the u32 to a i32
        let expected = [
            0xAA | 0xFF_00_00_00,
            0xAA_00 | 0xFF_00_00_00,
            0xAA_00_00 | 0xFF_00_00_00,
            0xFF,
            0xFF_00,
            0xFF_00_00,
        ];
        assert_eq!(
            buffer, expected,
            "\nResult:\n{:08x?}\nExpected:\n{:08x?}",
            &buffer, &expected
        );
    }
}
