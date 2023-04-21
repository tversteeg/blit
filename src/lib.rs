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

use imgref::{Img, ImgRefMut, ImgVec};
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
    fn to_blit_buffer_with_alpha(&self, alpha_treshold: u8) -> BlitBuffer;

    /// Convert the image to a custom `BlitBuffer` type which is optimized for applying the blitting operations.
    ///
    /// Ignore the alpha channel if set and use only a single color for transparency.
    fn to_blit_buffer_with_mask_color<C>(&self, mask_color: C) -> BlitBuffer
    where
        C: Into<Packed<Argb>>;

    /// Convert the image to a custom `BlitBuffer` type which is optimized for applying the blitting operations.
    ///
    /// Ignore the alpha channel if set and use only a single color for transparency.
    fn to_img_with_mask_color<C>(&self, mask_color: C) -> ImgVec<u32>
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
    pub area: Option<(i32, i32)>,

    /// Which part of the source buffer to render.
    ///
    /// - When `None` is used, `(0, 0, source_width, source_height)` is set instead.
    /// - With `Some(..)`, the values in the tuple are `(x, y, width, height)`.
    ///
    /// This is similar to UV coordinates but instead of relative positions in the range of `0..1` this takes absolute positions in the range `0..width` for horizontal positions and `0..height` for vertical positions.
    pub sub_rect: Option<(i32, i32, i32, i32)>,

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
    pub fn new<P>(position: P) -> Self
    where
        P: Into<(i32, i32)>,
    {
        let (x, y) = position.into();

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
        position: P,
        slice_left: i32,
        slice_right: i32,
        slice_top: i32,
        slice_bottom: i32,
    ) -> Self
    where
        P: Into<(i32, i32)>,
    {
        Self::new(position).with_slice9(slice_left, slice_right, slice_top, slice_bottom)
    }

    /// Set the size of the area `(width, height)` on the destination buffer.
    ///
    /// - When the area is smaller than the source buffer it effectively functions as the width and height parameters of [`BlitOptions::sub_rect`].
    /// - When the area is bigger than the source buffer the default behaviour will be tiling.
    ///
    /// # Sets field(s)
    ///
    /// - [`BlitOptions::area`]
    pub fn with_area<P>(mut self, area: P) -> Self
    where
        P: Into<(i32, i32)>,
    {
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
    pub fn with_sub_rect<P>(mut self, sub_rect: P) -> Self
    where
        P: Into<(i32, i32, i32, i32)>,
    {
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
        slice_left: i32,
        slice_right: i32,
        slice_top: i32,
        slice_bottom: i32,
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
    pub fn area<P>(&self, source_size: P) -> (i32, i32)
    where
        P: Into<(i32, i32)>,
    {
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
    pub fn set_sub_rect<P>(&mut self, sub_rect: P)
    where
        P: Into<(i32, i32, i32, i32)>,
    {
        let (x, y, width, height) = sub_rect.into();
        debug_assert!(x >= 0);
        debug_assert!(y >= 0);
        debug_assert!(width > 0);
        debug_assert!(height > 0);

        self.sub_rect = Some((x, y, width, height));

        // Don't tile the image when only the subrectangle is set
        if self.area.is_none() {
            self.area = Some((width, height));
        }
    }

    /// Get the source area sub rectangle `(x, y, width, height)`.
    ///
    /// - If [`BlitOptions::sub_rect`] is `None` the size of the source will be returned with `(0, 0)` as the position.
    /// - If [`BlitOptions::sub_rect`] and [`BlitOptions::area`] are set it, the `width` and `height` will be shrunk to match those of the area.
    pub fn sub_rect<P>(&self, source_size: P) -> (i32, i32, i32, i32)
    where
        P: Into<(i32, i32)>,
    {
        let (width, height) = source_size.into();

        // Get the sub rectangle defined or from the source
        let (sub_rect_x, sub_rect_y, sub_rect_width, sub_rect_height) =
            self.sub_rect.unwrap_or((0, 0, width, height));

        // The sub rectangle is never allowed to be bigger than the area
        let (sub_rect_width, sub_rect_height) = match self.area {
            Some((area_width, area_height)) => (
                sub_rect_width.min(area_width),
                sub_rect_height.min(area_height),
            ),
            None => (sub_rect_width, sub_rect_height),
        };

        (sub_rect_x, sub_rect_y, sub_rect_width, sub_rect_height)
    }

    /// Set the size of the area `(width, height)` on the destination buffer.
    ///
    /// - When the area is smaller than the source buffer it effectively functions as the width and height parameters of [`BlitOptions::sub_rect`].
    /// - When the area is bigger than the source buffer the default behaviour will be tiling.
    ///
    /// # Sets field(s)
    ///
    /// - [`BlitOptions::area`]
    pub fn set_area<P>(&mut self, area: P)
    where
        P: Into<(i32, i32)>,
    {
        let (width, height) = area.into();
        debug_assert!(width > 0);
        debug_assert!(height > 0);

        self.area = Some((width, height));
    }
}

/// Blit functions that can be called from multiple places.
pub trait Blit {
    /// Width and height of the source buffer in pixels.
    ///
    /// Changes depending on the state, for example with an animation it's the size of the current frame.
    ///
    /// This must be implemented so the other blit functions won't have to be implemented manually every time.
    fn size(&self) -> (i32, i32);

    fn blit_opt(&self, dst: &mut [u32], dst_width: usize, options: &BlitOptions) {
        self.blit(dst, dst_width, options.position());
    }

    /// Blit the image on a buffer.
    ///
    /// The complete image will be rendered, only clipped by the edges of the buffer size.
    fn blit(&self, dst: &mut [u32], dst_width: usize, (offset_x, offset_y): (i32, i32)) {
        let (width, height) = self.size();

        // We can blit the subrectangle with our entire size, effectively blitting everything
        self.blit_area(dst, dst_width, (offset_x, offset_y, width, height))
    }

    /// Blit a section of the image on a buffer.
    ///
    /// The sub rectangle is the section of the source that will be rendered.
    /// Its values are (X, Y, Width, Height) in pixels.
    /// A good mental model to keep for the section is that it's a view on the blit buffer that will be rendered.
    fn blit_subrect(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        (offset_x, offset_y): (i32, i32),
        sub_rect: (i32, i32, i32, i32),
    ) {
        let (_, _, sub_width, sub_height) = sub_rect;

        self.blit_area_subrect(
            dst,
            dst_width,
            (offset_x, offset_y, sub_width, sub_height),
            sub_rect,
        )
    }

    /// Blit in a specific area on a buffer.
    ///
    /// The sub rectangle is the section of the target that will be rendered.
    /// Its values are (X, Y, Width, Height) in pixels.
    ///
    /// For most types this will render a tiled view, for [`aseprite::Slice9BlitBuffer`] this will stretch the slices.
    fn blit_area(&self, dst: &mut [u32], dst_width: usize, area: (i32, i32, i32, i32)) {
        let (width, height) = self.size();

        // We can blit the sub rectangle but it's our entire size
        self.blit_area_subrect(dst, dst_width, area, (0, 0, width, height));
    }

    /// Blit a section of the image in a specific target area on buffer.
    ///
    /// The sub rectangle is the section of the target that will be rendered.
    /// Its values are (X, Y, Width, Height) in pixels.
    /// The sub rectangle is the section of the target that will be rendered.
    /// Its values are (X, Y, Width, Height) in pixels.
    ///
    /// For most types this will render a tiled view, for [`aseprite::Slice9BlitBuffer`] this will stretch the slices.
    fn blit_area_subrect(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        area: (i32, i32, i32, i32),
        sub_rect: (i32, i32, i32, i32),
    );
}

impl Blit for ImgVec<u32> {
    fn size(&self) -> (i32, i32) {
        (self.width() as i32, self.height() as i32)
    }

    fn blit_area_subrect(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        area: (i32, i32, i32, i32),
        sub_rect: (i32, i32, i32, i32),
    ) {
        todo!()
    }

    fn blit_opt(&self, dst_raw: &mut [u32], dst_width: usize, options: &BlitOptions) {
        let dst_height = dst_raw.len() / dst_width;
        // Convert the destination to an image so we can take a sub image from it
        let mut dst = ImgRefMut::new(dst_raw, dst_width, dst_height);

        let src_size = (self.width() as i32, self.height() as i32);

        let (width, height) = options.area(src_size);
        let dst_x = options.x.max(0);
        let dst_y = options.y.max(0);

        // Nothing to render when they fall outside of the screen
        if options.x <= -src_size.0
            || options.y <= -src_size.1
            || dst_x >= dst_width as i32
            || dst_y >= dst_height as i32
        {
            return;
        }

        // Subtract the difference from the size so we won't draw too many pixels
        let offset_x = dst_x - options.x;
        let offset_y = dst_y - options.y;

        let clamped_dst_width = (width - offset_x).clamp(0, dst_width as i32 - dst_x);
        let clamped_dst_height = (height - offset_y).clamp(0, dst.height() as i32 - dst_y);

        // Pixels on the destination we need to fill
        let mut target_dst = dst.sub_image_mut(
            dst_x as usize,
            dst_y as usize,
            clamped_dst_width as usize,
            clamped_dst_height as usize,
        );

        // Get the sub rectangle of the source to draw
        let (sub_rect_x, sub_rect_y, sub_rect_width, sub_rect_height) = options.sub_rect(src_size);

        // If the target size matches the source size we don't have to worry about tiling
        if width == sub_rect_width && height == sub_rect_height {
            // Pixels on the source we need to fill
            let sub_rect_x = sub_rect_x + offset_x;
            let sub_rect_y = sub_rect_y + offset_y;

            let target_src = self.sub_image(
                sub_rect_x as usize,
                sub_rect_y as usize,
                clamped_dst_width as usize,
                clamped_dst_height as usize,
            );

            target_dst
                .pixels_mut()
                .zip(target_src.pixels())
                .for_each(|(dst_pixel, src_pixel)| {
                    *dst_pixel = BlitBuffer::blit_pixel(*dst_pixel, src_pixel)
                });

            return;
        }

        // Draw the tiling
        let (x_tiles, y_tiles) = (width / sub_rect_width, height / sub_rect_height);

        // Draw the full tiles
        let mut new_options = BlitOptions::default();
        for y in 0..y_tiles {
            for x in 0..x_tiles {
                new_options.set_position((
                    options.x + x * sub_rect_width,
                    options.y + y * sub_rect_height,
                ));

                // Use the same sub rectangle for full tiles
                new_options.set_sub_rect((sub_rect_x, sub_rect_y, sub_rect_width, sub_rect_height));

                self.blit_opt(dst_raw, dst_width, &new_options);
            }
        }

        // How many pixels that are not a full sprite we still need to draw
        let (x_remainder, y_remainder) = (width % sub_rect_width, height % sub_rect_height);

        // Draw the right remainders
        if x_remainder > 0 {
            for y in 0..y_tiles {
                new_options.set_position((
                    options.x + x_tiles * sub_rect_width,
                    options.y + y * sub_rect_height,
                ));
                new_options.set_area((x_remainder, sub_rect_height));

                self.blit_opt(dst_raw, dst_width, &new_options);
            }
        }

        // Draw the bottom remainders
        if y_remainder > 0 {
            for x in 0..x_tiles {
                new_options.set_position((
                    options.x + x * sub_rect_width,
                    options.y + y_tiles * sub_rect_height,
                ));
                new_options.set_area((sub_rect_width, y_remainder));

                self.blit_opt(dst_raw, dst_width, &new_options);
            }
        }

        // Draw the single remaining corner
        if x_remainder > 0 && y_remainder > 0 {
            new_options.set_position((
                options.x + x_tiles * sub_rect_width,
                options.y + y_tiles * sub_rect_height,
            ));
            new_options.set_area((x_remainder, y_remainder));

            self.blit_opt(dst_raw, dst_width, &new_options);
        }
    }
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

    /// Blit the source buffer on the destination buffer.
    ///
    /// Only the width of the destination buffer needs to be supplied, the height is automatically calculated with `dst.len() / dst_width`.
    pub fn blit_opt(&self, dst: &mut [u32], dst_width: usize, options: &BlitOptions) {
        todo!()
    }

    /// Get the width of the buffer in pixels.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Get the height of the buffer in pixels.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Get a reference to the pixel data.
    pub fn pixels(&self) -> &[u32] {
        &self.data
    }

    /// Get a mutable reference to the pixel data.
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.data
    }

    /// Blit a sub rectangle clipped by an external rectangle.
    fn blit_clipped(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        (offset_x, offset_y): (i32, i32),
        (sub_rect_x, sub_rect_y, sub_rect_width, sub_rect_height): (i32, i32, i32, i32),
        (external_x, external_y, external_width, external_height): (i32, i32, i32, i32),
    ) {
        let dst_height = dst.len() / dst_width;

        let dst_width_i32 = dst_width as i32;
        let dst_height_i32 = dst_height as i32;

        // Ignore out of bounds images
        if offset_x > dst_width_i32 || offset_y > dst_height_i32 {
            return;
        }

        // Region we need to draw in
        let dst_start_x = offset_x.max(external_x).max(0);
        let dst_start_y = offset_y.max(external_y).max(0);
        let dst_end_x = (offset_x + sub_rect_width)
            .min(dst_width_i32)
            .min(external_x + external_width) as usize;
        let dst_end_y = (offset_y + sub_rect_height)
            .min(dst_height_i32)
            .min(external_y + external_height) as usize;
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

    /*
    /// Get an iterator of references slices for the sub image.
    fn sub_image_iter(
        &self,
        (x, y, width, height): (i32, i32, i32, i32),
    ) -> impl Iterator<Item = &[Color]> {
        // If the sub image is out of bounds don't return it
        if x <= -width || y <= -height || x > self.width || y > self.height {
            return std::iter::empty();
        }
    }
    */

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
        ImageVie
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
    fn blit(&self, dst: &mut [u32], dst_width: usize, (offset_x, offset_y): (i32, i32)) {
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
        self.blit_subrect(
            dst,
            dst_width,
            (offset_x, offset_y),
            (0, 0, self.width, self.height),
        )
    }

    fn blit_area_subrect(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        area: (i32, i32, i32, i32),
        sub_rect: (i32, i32, i32, i32),
    ) {
        let (area_x, area_y, area_width, area_height) = area;
        let (_sub_x, _sub_y, sub_width, sub_height) = sub_rect;

        // How much items we need to draw completely
        let full_x = area_width / sub_width;
        let full_y = area_height / sub_height;

        // What remaining part of the image we need to draw
        let remainder_x = area_x + full_x * sub_width;
        let remainder_y = area_y + full_y * sub_height;

        for y in 0..full_y {
            // Position on the buffer to render this image
            let target_y = area_y + y * sub_height;

            for x in 0..full_x {
                // Position on the buffer to render this image
                let target_x = area_x + x * sub_width;

                // Blit the completely filled rectangles
                self.blit_clipped(dst, dst_width, (target_x, target_y), sub_rect, area);
            }

            // Blit the vertical remainders
            self.blit_clipped(dst, dst_width, (remainder_x, target_y), sub_rect, area);
        }

        for x in 0..full_x {
            // Position on the buffer to render this image
            let target_x = area_x + x * sub_width;

            // Blit the horizontal remainders
            self.blit_clipped(dst, dst_width, (target_x, remainder_y), sub_rect, area);
        }

        // Blit the single corner remainder
        self.blit_clipped(dst, dst_width, (remainder_x, remainder_y), sub_rect, area);
    }

    fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn blit_opt(&self, dst_raw: &mut [u32], dst_width: usize, options: &BlitOptions) {
        let dst_height = dst_raw.len() / dst_width;
        // Convert the destination to view so we can calculate with it
        let mut dst = match ImageView::new_unchecked((0, 0, dst_width, dst_height)) {
            Some(view) => view,
            None => return,
        };

        // Convert our source to a view
        let mut src = match ImageView::new_unchecked((0, 0, dst_width, dst_height)) {
            Some(view) => view,
            None => return,
        };

        let src_size = (self.width() as i32, self.height() as i32);

        let (width, height) = options.area(src_size);
        let dst_x = options.x.max(0);
        let dst_y = options.y.max(0);

        // Nothing to render when they fall outside of the screen
        if options.x <= -src_size.0
            || options.y <= -src_size.1
            || dst_x >= dst_width as i32
            || dst_y >= dst_height as i32
        {
            return;
        }

        // Subtract the difference from the size so we won't draw too many pixels
        let offset_x = dst_x - options.x;
        let offset_y = dst_y - options.y;

        let clamped_dst_width = (width - offset_x).clamp(0, dst_width as i32 - dst_x);
        let clamped_dst_height = (height - offset_y).clamp(0, dst.height() as i32 - dst_y);

        // Pixels on the destination we need to fill
        let mut target_dst = dst.sub_image_mut(
            dst_x as usize,
            dst_y as usize,
            clamped_dst_width as usize,
            clamped_dst_height as usize,
        );

        // Get the sub rectangle of the source to draw
        let (sub_rect_x, sub_rect_y, sub_rect_width, sub_rect_height) = options.sub_rect(src_size);

        // If the target size matches the source size we don't have to worry about tiling
        if width == sub_rect_width && height == sub_rect_height {
            // Pixels on the source we need to fill
            let sub_rect_x = sub_rect_x + offset_x;
            let sub_rect_y = sub_rect_y + offset_y;

            let target_src = self.sub_image(
                sub_rect_x as usize,
                sub_rect_y as usize,
                clamped_dst_width as usize,
                clamped_dst_height as usize,
            );

            target_dst
                .pixels_mut()
                .zip(target_src.pixels())
                .for_each(|(dst_pixel, src_pixel)| {
                    *dst_pixel = BlitBuffer::blit_pixel(*dst_pixel, src_pixel)
                });

            return;
        }

        // Draw the tiling
        let (x_tiles, y_tiles) = (width / sub_rect_width, height / sub_rect_height);

        // Draw the full tiles
        let mut new_options = BlitOptions::default();
        for y in 0..y_tiles {
            for x in 0..x_tiles {
                new_options.set_position((
                    options.x + x * sub_rect_width,
                    options.y + y * sub_rect_height,
                ));

                // Use the same sub rectangle for full tiles
                new_options.set_sub_rect((sub_rect_x, sub_rect_y, sub_rect_width, sub_rect_height));

                self.blit_opt(dst_raw, dst_width, &new_options);
            }
        }

        // How many pixels that are not a full sprite we still need to draw
        let (x_remainder, y_remainder) = (width % sub_rect_width, height % sub_rect_height);

        // Draw the right remainders
        if x_remainder > 0 {
            for y in 0..y_tiles {
                new_options.set_position((
                    options.x + x_tiles * sub_rect_width,
                    options.y + y * sub_rect_height,
                ));
                new_options.set_area((x_remainder, sub_rect_height));

                self.blit_opt(dst_raw, dst_width, &new_options);
            }
        }

        // Draw the bottom remainders
        if y_remainder > 0 {
            for x in 0..x_tiles {
                new_options.set_position((
                    options.x + x * sub_rect_width,
                    options.y + y_tiles * sub_rect_height,
                ));
                new_options.set_area((sub_rect_width, y_remainder));

                self.blit_opt(dst_raw, dst_width, &new_options);
            }
        }

        // Draw the single remaining corner
        if x_remainder > 0 && y_remainder > 0 {
            new_options.set_position((
                options.x + x_tiles * sub_rect_width,
                options.y + y_tiles * sub_rect_height,
            ));
            new_options.set_area((x_remainder, y_remainder));

            self.blit_opt(dst_raw, dst_width, &new_options);
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
