//! Draw sprites quickly using bitwise operations and a masking color.
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

use palette::{rgb::channels::Argb, IntoColor, Packed};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "image")]
pub mod image_feature;
#[cfg(feature = "image")]
pub use image_feature::*;
#[cfg(feature = "aseprite")]
pub mod aseprite_feature;
#[cfg(feature = "aseprite")]
pub use aseprite_feature::*;

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
        Self::from_iter(src.iter().map(|pixel| *pixel), width, alpha_treshold)
    }

    /// Create a instance from a iterator of ARGB data packed in a single `u32`.
    ///
    /// It's assumed that the alpha channel in the resulting pixel is properly set.
    /// The alpha treshold is the offset point at which an alpha value will be used as either a transparent pixel or a colored one.
    pub fn from_iter<I>(iter: I, width: i32, alpha_treshold: u8) -> Self
    where
        I: Iterator<Item = Color>,
    {
        let alpha_treshold = alpha_treshold as Color;

        // Create the data buffer filled with transparent pixels
        let data = iter
            .map(|pixel| {
                let alpha = pixel >> 24;
                if alpha < alpha_treshold {
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
    pub fn blit(&self, dst: &mut [u32], dst_width: usize, offset: (i32, i32)) {
        let src_size = (self.width, self.height);
        let dst_size = (dst_width as i32, (dst.len() / dst_width) as i32);

        if offset == (0, 0) && dst_size == src_size {
            // If the sizes match and the buffers are aligned we don't have to do any special
            // bounds checks
            dst.iter_mut()
                .zip(&self.data[..])
                .for_each(|(pixel, color)| {
                    *pixel = blit(*pixel, *color);
                });

            return;
        }

        let dst_start = (offset.0.max(0), offset.1.max(0));
        let dst_end = (
            (offset.0 + src_size.0).min(dst_size.0),
            (offset.1 + src_size.1).min(dst_size.1),
        );

        for dst_y in dst_start.1..dst_end.1 {
            let src_y = dst_y - offset.1;

            let dst_y_index = dst_y * dst_size.0;
            let src_y_index = src_y * src_size.0;

            for dst_x in dst_start.0..dst_end.0 {
                let src_x = dst_x - offset.0;

                let src_index = (src_x + src_y_index) as usize;
                let dst_index = (dst_x + dst_y_index) as usize;

                // First draw the mask as black on the background using an AND operation, and then
                // draw the colors using an OR operation
                dst[dst_index] = blit(dst[dst_index], self.data[src_index]);
            }
        }
    }

    /// Blit a section of the image on a buffer.
    pub fn blit_rect(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        offset: (i32, i32),
        sub_rect: (i32, i32, i32, i32),
    ) {
        let dst_size = (dst_width as i32, (dst.len() / dst_width) as i32);

        let src_size = (self.width, self.height);

        let dst_start = (std::cmp::max(offset.0, 0), std::cmp::max(offset.1, 0));
        let dst_end = (
            std::cmp::min(offset.0 + sub_rect.2, dst_size.0),
            std::cmp::min(offset.1 + sub_rect.3, dst_size.1),
        );

        for dst_y in dst_start.1..dst_end.1 {
            let src_y = dst_y - offset.1 + sub_rect.1;

            let dst_y_index = dst_y * dst_size.0;
            let src_y_index = src_y * src_size.0;

            for dst_x in dst_start.0..dst_end.0 {
                let src_x = dst_x - offset.0 + sub_rect.0;

                let src_index = (src_x + src_y_index) as usize;
                let dst_index = (dst_x + dst_y_index) as usize;

                // First draw the mask as black on the background using an AND operation, and then
                // draw the colors using an OR operation
                let color = self.data[src_index];
                dst[dst_index] = blit(dst[dst_index], color);
            }
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
}

impl std::fmt::Debug for BlitBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlitBuffer")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

/// The main logic of calculating the resulting color that needs to be drawn.
#[inline(always)]
fn blit(source: Color, color: Color) -> Color {
    // Get the mask of the color as a single "u8"
    let mask = color >> 24;
    // Fill the RGB channel with the mask
    let rgb_mask = (mask << 24) | (mask << 16) | (mask << 8) | mask;

    // Get the inverse of the mask, but keep the alpha bits
    let rgb_mask_inverse = rgb_mask ^ 0xFF_FF_FF;

    // Either get the source color when the blit pixel is masked, or get the blit pixel when it isn't
    // Always set the transparency
    (source & rgb_mask_inverse) | (color & rgb_mask) | 0xFF_00_00_00
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
            0xFF | 0xFF_00_00_00,
            0xFF_00 | 0xFF_00_00_00,
            0xFF_00_00 | 0xFF_00_00_00,
        ];
        assert_eq!(
            buffer, expected,
            "\nResult:\n{:08x?}\nExpected:\n{:08x?}",
            &buffer, &expected
        );
    }
}
