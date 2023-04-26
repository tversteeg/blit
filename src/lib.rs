//! Draw sprites quickly using a masking color or an alpha treshold.
//!
//! All colors can be constructed from the [`palette`](https://crates.io/crates/palette) crate or directly with an RGB `u32` where the alpha channel is ignored.
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "image")] mod test {
//! use blit::{Blit, ToBlitBuffer};
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
mod geom;
#[cfg(feature = "image")]
mod image;
pub mod slice;
mod view;

/// Commonly used imports.
///
/// ```rust
/// use blit::prelude::*;
/// ```
pub mod prelude {
    #[cfg(feature = "image")]
    pub use crate::ToBlitBuffer;
    pub use crate::{Blit, BlitBuffer};
}

use std::ops::Range;

use error::{Error, Result};
pub use geom::{Size, SubRect};
use num_traits::ToPrimitive;
use palette::{rgb::channels::Argb, Packed};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use slice::Slice;
use view::ImageView;

/// Internal representation of a color.
type Color = u32;

/// Add blitting functions to external image types.
///
/// Can be used to create a custom implementation if you want different image or other formats.
pub trait ToBlitBuffer {
    /// Convert the image to a custom `BlitBuffer` type which is optimized for applying the blitting operations.
    ///
    /// It's assumed that the alpha channel in the resulting pixel is properly set.
    /// The alpha treshold is the offset point at which an alpha value will be used as either a transparent pixel or a colored one.
    fn to_blit_buffer_with_alpha(&self, alpha_treshold: u8) -> Result<BlitBuffer>;

    /// Convert the image to a custom `BlitBuffer` type which is optimized for applying the blitting operations.
    ///
    /// Ignore the alpha channel if set and use only a single color for transparency.
    fn to_blit_buffer_with_mask_color<C>(&self, mask_color: C) -> Result<BlitBuffer>
    where
        C: Into<Packed<Argb>>;
}

/// Sprite blitting options.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BlitOptions {
    /// Horizontal position on the destination buffer.
    pub x: i32,

    /// Vertical position on the destination buffer.
    pub y: i32,

    /// Size of the area `(width, height)` on the destination buffer.
    ///
    /// - When `None` is used, the size of the source buffer or of the subrectangle if set will be used.
    /// - When the area is smaller than the source buffer it effectively functions as the width and height parameters of [`BlitOptions::sub_rect`].
    /// - When the area is bigger than the source buffer the default behaviour will be tiling.
    ///
    /// ```rust
    /// # use blit::BlitOptions;
    /// assert_eq!(
    ///     BlitOptions::default().with_area((10, 10)).sub_rect((100, 100)),
    ///     BlitOptions::default().with_sub_rect((0, 0, 10, 10)).sub_rect((100, 100))
    /// );
    /// ```
    pub area: Option<Size>,

    /// Which part of the source buffer to render.
    ///
    /// - When `None` is used, `(0, 0, source_width, source_height)` is set instead.
    /// - With `Some(..)`, the values in the tuple are `(x, y, width, height)`.
    ///
    /// This is similar to UV coordinates but instead of relative positions in the range of `0..1` this takes absolute positions in the range `0..width` for horizontal positions and `0..height` for vertical positions.
    pub sub_rect: Option<SubRect>,

    /// Divide the source buffer into multiple vertical sections and repeat the chosen section to fill the area.
    ///
    /// This is only used when [`BlitOptions::area`] is set.
    pub vertical_slices: Option<Slice>,

    /// Divide the source buffer into multiple horizontal sections and repeat the chosen section to fill the area.
    ///
    /// This is only used when [`BlitOptions::area`] is set.
    pub horizontal_slices: Option<Slice>,
}

impl BlitOptions {
    /// Setup options for blitting at position `(x, y)`.
    ///
    /// When no other fields are changed or methods are called this will render the full source.
    pub fn new<P>(x: P, y: P) -> Self
    where
        P: ToPrimitive,
    {
        let (x, y) = (x.to_i32().unwrap_or(0), y.to_i32().unwrap_or(0));

        Self {
            x,
            y,
            ..Default::default()
        }
    }

    /// Setup options for blitting at position `(x, y)`.
    ///
    /// When no other fields are changed or methods are called this will render the full source.
    pub fn from_tuple<P>((x, y): (P, P)) -> Self
    where
        P: ToPrimitive,
    {
        let (x, y) = (x.to_i32().unwrap_or(0), y.to_i32().unwrap_or(0));

        Self {
            x,
            y,
            ..Default::default()
        }
    }

    /// Draw as a scalable [9-slice graphic](https://en.wikipedia.org/wiki/9-slice_scaling).
    ///
    /// # Sets field(s)
    ///
    /// - [`BlitOptions::x`]
    /// - [`BlitOptions::y`]
    /// - [`BlitOptions::vertical_slices`]
    /// - [`BlitOptions::horizontal_slices`].
    ///
    /// All other fields will be set to [`BlitOptions::default()`].
    pub fn slice9<P>(
        x: P,
        y: P,
        slice_left: u32,
        slice_right: u32,
        slice_top: u32,
        slice_bottom: u32,
    ) -> Self
    where
        P: ToPrimitive,
    {
        Self::new(x.to_i32().unwrap_or(0), y.to_i32().unwrap_or(0)).with_slice9(
            slice_left,
            slice_right,
            slice_top,
            slice_bottom,
        )
    }

    /// Set the size of the area `(width, height)` on the destination buffer.
    ///
    /// - When the area is smaller than the source buffer it effectively functions as the width and height parameters of [`BlitOptions::sub_rect`].
    /// - When the area is bigger than the source buffer the default behaviour will be tiling.
    ///
    /// # Sets field(s)
    ///
    /// - [`BlitOptions::area`]
    pub fn with_area(mut self, area: Size) -> Self {
        self.set_area(area);

        self
    }

    /// Set which part of the source buffer to render.
    ///
    /// - When `None` is used, `(0, 0, source_width, source_height)` is set instead.
    /// - With `Some(..)`, the values in the tuple are `(x, y, width, height)`.
    ///
    /// This is similar to UV coordinates but instead of relative positions in the range of `0..1` this takes absolute positions in the range `0..width` for horizontal positions and `0..height` for vertical positions.
    ///
    /// # Sets field(s)
    ///
    /// - [`BlitOptions::sub_rect`]
    /// - [`BlitOptions::area`] to `(width, height)` if it's `None`
    pub fn with_sub_rect(mut self, sub_rect: SubRect) -> Self {
        self.set_sub_rect(sub_rect);

        self
    }

    /// Draw as a scalable [9-slice graphic](https://en.wikipedia.org/wiki/9-slice_scaling).
    ///
    /// # Sets field(s)
    ///
    /// - [`BlitOptions::vertical_slices`]
    /// - [`BlitOptions::horizontal_slices`].
    pub fn with_slice9(
        mut self,
        slice_left: u32,
        slice_right: u32,
        slice_top: u32,
        slice_bottom: u32,
    ) -> Self {
        self.vertical_slices = Some(Slice::ternary_middle(slice_left, slice_right));
        self.horizontal_slices = Some(Slice::ternary_middle(slice_top, slice_bottom));

        self
    }

    /// Set the position `(x, y)`.
    ///
    /// # Sets field(s)
    ///
    /// - [`BlitOptions::x`]
    /// - [`BlitOptions::y`].
    pub fn set_position<P>(&mut self, position: P)
    where
        P: Into<(i32, i32)>,
    {
        let (x, y) = position.into();

        self.x = x;
        self.y = y;
    }

    /// Get the position `(x, y)`.
    pub fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    /// Get the destination area `(width, height)`.
    ///
    /// If [`BlitOptions::area`] is `None` the size of the source will be returned.
    pub fn area(&self, source_size: Size) -> Size {
        self.area.unwrap_or(source_size.into())
    }

    /// Set which part of the source buffer to render.
    ///
    /// - When `None` is used, `(0, 0, source_width, source_height)` is set instead.
    /// - With `Some(..)`, the values in the tuple are `(x, y, width, height)`.
    ///
    /// This is similar to UV coordinates but instead of relative positions in the range of `0..1` this takes absolute positions in the range `0..width` for horizontal positions and `0..height` for vertical positions.
    ///
    /// # Sets field(s)
    ///
    /// - [`BlitOptions::sub_rect`]
    /// - [`BlitOptions::area`] to `(width, height)` if it's `None`
    pub fn set_sub_rect(&mut self, sub_rect: SubRect) {
        self.sub_rect = Some(sub_rect);

        // Don't tile the image when only the subrectangle is set
        if self.area.is_none() {
            self.area = Some(sub_rect.size);
        }
    }

    /// Get the source area sub rectangle `(x, y, width, height)`.
    ///
    /// - If [`BlitOptions::sub_rect`] is `None` the size of the source will be returned with `(0, 0)` as the position.
    /// - If [`BlitOptions::sub_rect`] and [`BlitOptions::area`] are set it, the `width` and `height` will be shrunk to match those of the area.
    pub fn sub_rect(&self, source_size: Size) -> SubRect {
        // Get the sub rectangle defined or from the source
        let mut sub_rect = self
            .sub_rect
            .unwrap_or_else(|| SubRect::from_size(source_size));

        // The sub rectangle is never allowed to be bigger than the area
        let sub_rect_size = match self.area {
            Some(area) => sub_rect.size.min(area),
            None => sub_rect.size,
        };

        sub_rect.size = sub_rect_size;

        sub_rect
    }

    /// Set the size of the area `(width, height)` on the destination buffer.
    ///
    /// - When the area is smaller than the source buffer it effectively functions as the width and height parameters of [`BlitOptions::sub_rect`].
    /// - When the area is bigger than the source buffer the default behaviour will be tiling.
    ///
    /// # Sets field(s)
    ///
    /// - [`BlitOptions::area`]
    pub fn set_area(&mut self, area: Size) {
        self.area = Some(area.into());
    }
}

/// Blit functions that can be called from multiple places.
pub trait Blit {
    fn blit(&self, dst: &mut [u32], dst_size: Size, options: &BlitOptions);
}

/// A data structure holding a color and a mask buffer to make blitting on a buffer real fast.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct BlitBuffer {
    /// Image size in pixels.
    size: Size,

    /// Vector of colors, the highest 8 bits are alpha and the remaining 24 bits the RGB color channels.
    data: Vec<Color>,
}

impl BlitBuffer {
    /// Create a instance from a buffer of ARGB data packed in a single `u32`.
    ///
    /// It's assumed that the alpha channel in the resulting pixel is properly set.
    /// The alpha treshold is the offset point at which an alpha value will be used as either a transparent pixel or a colored one.
    pub fn from_buffer<S>(src: &[Color], width: S, alpha_treshold: u8) -> Result<Self>
    where
        S: ToPrimitive,
    {
        Self::from_iter(src.iter().copied(), width, alpha_treshold)
    }

    /// Create a instance from a iterator of ARGB data packed in a single `u32`.
    ///
    /// It's assumed that the alpha channel in the resulting pixel is properly set.
    /// The alpha treshold is the offset point at which an alpha value will be used as either a transparent pixel or a colored one.
    pub fn from_iter<I, S>(iter: I, width: S, alpha_treshold: u8) -> Result<Self>
    where
        I: Iterator<Item = Color>,
        S: ToPrimitive,
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
        let size = Size::from_len(data.len(), width.to_usize().unwrap_or_default());

        Ok(Self { size, data })
    }

    /// Width of the buffer in pixels.
    pub fn width(&self) -> u32 {
        self.size.width
    }

    /// Height of the buffer in pixels.
    pub fn height(&self) -> u32 {
        self.size.height
    }

    /// Size of the blitbuffer in pixels.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get a reference to the pixel data.
    pub fn pixels(&self) -> &[Color] {
        &self.data
    }

    /// Get a mutable reference to the pixel data.
    pub fn pixels_mut(&mut self) -> &mut [Color] {
        &mut self.data
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

    /// Get a horizontal view based on this whole image.
    fn view(&self) -> ImageView {
        ImageView::full(self.size)
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

impl Blit for BlitBuffer {
    fn blit(&self, dst: &mut [u32], dst_size: Size, options: &BlitOptions) {
        // Convert the destination to view so we can calculate with it
        let dst = ImageView::full(dst_size);

        // Convert our source to a view
        let src = ImageView::full(self.size);

        // Find a view on the dst based on the area
        let area = options.area(self.size);
        let dst_area = match dst.sub_i32(options.x, options.y, area) {
            Some(dst_area) => dst_area,
            None => return,
        };

        // Another view based on the subrectangle
        let sub_rect_view = match src.sub(options.sub_rect(self.size)) {
            Some(sub_rect_view) => sub_rect_view,
            None => return,
        };

        // We can draw the image exactly
        if sub_rect_view.size() == area {
            // Pixel range of the source
            sub_rect_view
                .parent_ranges_iter(src.width() as usize)
                // Zipped with pixel range of the destination
                .zip(dst_area.parent_ranges_iter(dst_size.width as usize))
                .for_each(|range| todo!());
        }

        dbg!(dst_area);
    }
}

impl std::fmt::Debug for BlitBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlitBuffer")
            .field("width", &self.size.width)
            .field("height", &self.size.height)
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
            u32::new(2).unwrap(),
            127,
        )
        .unwrap();
        blit.blit(
            &mut buffer,
            Size::new(2, 3).unwrap(),
            &BlitOptions::new((0, 0)),
        );

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
