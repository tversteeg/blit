//! Draw sprites quickly using a masking color or an alpha treshold.
//!
//! This crate works with RGBA `u32` buffers.
//! The alpha channel can only be read with a singular treshold, converting it to a binary transparent or opaque color.
//! The reason this limitation is in place is that it allows efficient rendering optimizations.
//!
//! For ergonomic use of this crate without needing to type convert everything most functions accepting numbers are generic with the number types being [`num_traits::ToPrimitive`], this might seem confusing but any number can be passed to these functions immediately.
//!
//! When using this crate the most important function to know about is [`Blit::blit`], which is implemented for [`BlitBuffer`].
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "image")] mod test {
//! use blit::{Blit, ToBlitBuffer, geom::Size};
//!
//! const CANVAS_SIZE: Size = Size { width: 180, height: 180 };
//! const MASK_COLOR: u32 = 0xFF_00_FF;

//! # fn main()  {
//! // Create a buffer in which we'll draw our image
//! let mut canvas: Vec<u32> = vec![0xFF_FF_FF_FF; CANVAS_SIZE.pixels()];
//!
//! // Load the image from disk using the `image` crate
//! let img = image::open("examples/smiley_rgb.png").unwrap().into_rgb8();
//!
//! // Blit by creating a special blitting buffer first where the MASK_COLOR will be the color that will be made transparent
//! let blit_buffer = img.to_blit_buffer_with_mask_color(MASK_COLOR);
//!
//! // Draw the image 2 times to the buffer
//! blit_buffer.blit(&mut canvas, CANVAS_SIZE).position((10, 10)).draw();
//! blit_buffer.blit(&mut canvas, CANVAS_SIZE).position((20, 20)).draw();
//! # }}
//! ```

pub mod geom;
#[cfg(feature = "image")]
mod image;
pub mod ops;
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
    pub use crate::{
        geom::{Coordinate, Rect, Size},
        slice::Slice,
        Blit, BlitBuffer,
    };
}

use geom::{Coordinate, Rect, Size};

use std::ops::Range;

use num_traits::ToPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use slice::{Slice, SliceProjection};
use view::ImageView;

/// Internal representation of a color.
type Color = u32;

/// Blit functions that can be called from multiple places.
pub trait Blit {
    /// Size of the source buffer.
    fn size(&self) -> Size;

    /// Start a blitting pipeline.
    fn blit<'a, 'b, S>(
        &'a self,
        target: &'b mut [u32],
        target_size: S,
    ) -> BlitPipeline<'a, 'b, Self>
    where
        S: Into<Size>,
        Self: Sized,
    {
        BlitPipeline::new(self, target, target_size.into())
    }

    /// Library function for crates implementing this trait.
    ///
    /// This function shouldn't be called directly.
    fn blit_impl(
        &self,
        target: &mut [Color],
        target_width: usize,
        x: usize,
        y: usize,
        u: usize,
        v: usize,
        width: usize,
        height: usize,
    );
}

/// Convert external image types to a specialized buffer optimized for blitting.
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
    fn to_blit_buffer_with_mask_color(&self, mask_color: u32) -> BlitBuffer;
}

/// Pipeline for rendering a source image on a target image.
///
/// This pipeline can be started by calling [`Blit::blit`] on [`BlitBuffer`] which can be created by implementations of the [`ToBlitBuffer`] trait.
#[derive(Debug)]
pub struct BlitPipeline<'a, 'b, B: Blit> {
    /// What to blit.
    source: &'a B,
    /// View on the source buffer.
    source_view: ImageView,

    /// Where to blit it to.
    target: &'b mut [Color],
    /// Full size of the target buffer.
    target_size: Size,
    /// View on the target buffer.
    target_view: ImageView,
}

impl<'a, 'b, B: Blit> BlitPipeline<'a, 'b, B> {
    /// Set the render position on the target `(x, y)`.
    #[must_use = "call `.draw()` to blit"]
    pub fn position<P>(mut self, position: P) -> Self
    where
        P: Into<Coordinate>,
    {
        let position = position.into();

        // Shift the image around in the target rectangle
        let (uv_offset, target_view) = self.target_view.shift(position.x, position.y);
        self.target_view = target_view;

        // Make the target view the same size as the source view
        self.target_view.shrink(self.source_view.size());

        // Shift the source by the shifted UV coordinates
        self.source_view.add_coordinate_abs(uv_offset);

        self
    }

    /// Set the size of the area `(width, height)` on the destination buffer.
    ///
    /// - When the area is smaller than the source buffer it effectively functions as the width and height parameters of [`BlitOptions::sub_rect`].
    /// - When the area is bigger than the source buffer the default behaviour will be tiling.
    #[must_use = "call `.draw()` to blit"]
    pub fn area<S>(mut self, area: S) -> Self
    where
        S: Into<Size>,
    {
        self
    }

    /// Offset the position on the source view.
    #[must_use = "call `.draw()` to blit"]
    pub fn uv<C>(mut self, uv: C) -> Self
    where
        C: Into<Coordinate>,
    {
        self
    }

    /// Set which part of the source buffer to render.
    #[must_use = "call `.draw()` to blit"]
    pub fn sub_rect<R>(mut self, sub_rect: R) -> Self
    where
        R: Into<Rect>,
    {
        self
    }

    /// Set which part of the target buffer to render.
    #[must_use = "call `.draw()` to blit"]
    pub fn mask<R>(mut self, mask: R) -> Self
    where
        R: Into<Rect>,
    {
        self
    }

    /// Draw as a scalable [9-slice graphic](https://en.wikipedia.org/wiki/9-slice_scaling).
    ///
    /// The sub-rectangle of the center piece that will be scaled needs to be passed.
    /// Note that the rectangle has a width and a height instead of the absolute coordinates the other slice functions accept.
    #[must_use = "call `.draw()` to blit"]
    pub fn slice9<R>(mut self, center: R) -> Self
    where
        R: Into<Rect>,
    {
        self
    }

    /// Scale a single horizontal piece of the buffer while keeping the other parts the same height.
    #[must_use = "call `.draw()` to blit"]
    pub fn horizontal_slice(mut self, slice: Slice) -> Self {
        self
    }

    /// Scale a single vertical piece of the buffer while keeping the other parts the same height.
    #[must_use = "call `.draw()` to blit"]
    pub fn vertical_slice(mut self, slice: Slice) -> Self {
        self
    }

    /// Render the result.
    pub fn draw(&mut self) {
        assert!(self.source_view.x() >= 0);
        assert!(self.source_view.y() >= 0);
        assert!(self.target_view.x() >= 0);
        assert!(self.target_view.y() >= 0);
        assert!(self.target_view.x() as u32 + self.target_view.width() <= self.target_size.width);
        assert!(self.target_view.y() as u32 + self.target_view.height() <= self.target_size.height);

        self.source.blit_impl(
            &mut self.target,
            self.target_size.width as usize,
            self.target_view.x() as usize,
            self.target_view.y() as usize,
            self.source_view.x() as usize,
            self.source_view.y() as usize,
            self.target_view.width() as usize,
            self.target_view.height() as usize,
        );
    }

    /// Construct a new pipeline with defaults.
    pub(crate) fn new(source: &'a B, target: &'b mut [Color], target_size: Size) -> Self {
        let pos = Coordinate::new(0, 0);

        let target_view = ImageView::new_unchecked(0, 0, target_size);
        let source_view = ImageView::new_unchecked(0, 0, source.size());

        Self {
            source,
            source_view,
            target,
            target_size,
            target_view,
        }
    }
}

/// A pipeline that's been split over multiple separate rendering parts by operations.
pub struct SplitPipeline<'a, 'b, B: Blit>(Vec<BlitPipeline<'a, 'b, B>>);

impl<'a, 'b, B: Blit> SplitPipeline<'a, 'b, B> {
    /// Render the result.
    pub fn draw(&mut self) {
        self.0.iter_mut().for_each(BlitPipeline::draw);
    }
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
    /// Create a instance from a buffer of RGBA data packed in a single `u32`.
    ///
    /// It's assumed that the alpha channel in the resulting pixel is properly set.
    /// The alpha treshold is the offset point at which an alpha value will be used as either a transparent pixel or a colored one.
    #[must_use]
    pub fn from_buffer<S>(src: &[Color], width: S, alpha_treshold: u8) -> Self
    where
        S: ToPrimitive,
    {
        Self::from_iter(src.iter().copied(), width, alpha_treshold)
    }

    /// Create a instance from a iterator of RGBA data packed in a single `u32`.
    ///
    /// It's assumed that the alpha channel in the resulting pixel is properly set.
    /// The alpha treshold is the offset point at which an alpha value will be used as either a transparent pixel or a colored one.
    #[must_use]
    pub fn from_iter<I, S>(iter: I, width: S, alpha_treshold: u8) -> Self
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

        Self { size, data }
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

    /*
    /// Divide the target area into given slices of rectangles to draw.
    ///
    /// A `(source, target)` rectangle tuple is returned.
    fn slice_projections(
        &self,
        options: &BlitOptions,
        target_area: Size,
    ) -> Vec<(SubRect, SubRect)> {
        match (options.vertical_slice, options.horizontal_slice) {
            // No slices, so no need to split it
            (None, None) => Vec::new(),
            // Only a horizontal slice
            (None, Some(horizontal)) => horizontal
                .divide_area_iter(self.height(), target_area.height)
                .map(|horizontal| horizontal.into_sub_rects_static_x(self.width()))
                .collect(),
            // Only a vertical slice
            (Some(vertical), None) => vertical
                .divide_area_iter(self.width(), target_area.width)
                .map(|vertical| vertical.into_sub_rects_static_y(self.height()))
                .collect(),
            // The buffer is split both horizontally and vertically
            (Some(vertical), Some(horizontal)) => {
                let horizontal_ranges = vertical
                    .divide_area_iter(self.width(), target_area.width)
                    .collect::<Vec<_>>();
                let vertical_ranges =
                    horizontal.divide_area_iter(self.height(), target_area.height);

                // Return a cartesian product of all ranges
                vertical_ranges
                    .flat_map(|vertical| {
                        horizontal_ranges.iter().map(move |horizontal| {
                            SliceProjection::combine_into_sub_rects(horizontal, &vertical)
                        })
                    })
                    .collect()
            }
        }
    }
    */

    /*
    /// Blit a sliced section.
    fn blit_slice(&self, dst: &mut [u32], dst_size: Size, options: &BlitOptions) {
        // If the size of the image is the same as our buffer and the location is zero we can completely blit all bytes
        if options.x == 0 && options.y == 0 && dst_size == self.size && options.mask.is_none() {
            let pixels = dst_size.pixels();
            self.blit_horizontal(dst, 0..pixels, 0..pixels);

            return;
        }

        // Convert the destination to view so we can calculate with it
        let mut dst_view = ImageView::full(dst_size);

        // Take the mask view if applicable and translate the position
        let (x, y) = if let Some(mask) = options.mask {
            if let Some(masked_view) = dst_view.sub(mask) {
                // Translate the position relative to the new view
                let x = options.x - mask.x;
                let y = options.y - mask.y;
                dbg!(x, y);

                // Set the new view
                dst_view = masked_view;

                (x, y)
            } else {
                // Nothing to render
                return;
            }
        } else {
            (options.x, options.y)
        };

        // Convert our source to a view
        let src_view = ImageView::full(self.size);

        // Find a view on the dst based on the area
        let area = options.area(self.size);
        let mut dst_area = match dst_view.sub_i32(options.x, options.y, area) {
            Some(dst_area) => dst_area,
            None => return,
        };

        // Another view based on the subrectangle
        let sub_rect_view = match src_view.sub(options.sub_rect(self.size)) {
            Some(sub_rect_view) => sub_rect_view,
            None => return,
        };

        // We can draw the image exactly
        if sub_rect_view.size() == area {
            // Pixel range of the source
            sub_rect_view
                .parent_ranges_iter(self.size)
                // Zipped with pixel range of the destination
                .zip(dst_area.parent_ranges_iter(dst_size))
                .for_each(|(src_range, dst_range)| self.blit_horizontal(dst, dst_range, src_range));
        } else {
            // Recursively call this function with a new area defined by the sub rectangle to tile

            // Amount of tiles we need to fully render
            let tiles = area / sub_rect_view.size();
            let remainder = area % sub_rect_view.size();

            for tile_x in 0..tiles.width {
                // Fully render the filled tiles
                for tile_y in 0..tiles.height {
                    let options = BlitOptions::new_position(
                        x + (tile_x * sub_rect_view.width()) as i32,
                        y + (tile_y * sub_rect_view.height()) as i32,
                    )
                    .with_sub_rect(sub_rect_view.as_sub_rect());

                    self.blit_slice(dst, dst_size, &options);
                }

                if remainder.height > 0 {
                    // Render the horizontal remainder
                    let options = BlitOptions::new_position(
                        x + (tile_x * sub_rect_view.width()) as i32,
                        y + (tiles.height * sub_rect_view.height()) as i32,
                    )
                    .with_sub_rect(sub_rect_view.as_sub_rect())
                    .with_area((sub_rect_view.width(), remainder.height));

                    self.blit_slice(dst, dst_size, &options);
                }
            }

            if remainder.width > 0 {
                // Render the vertical remainder
                for tile_y in 0..tiles.height {
                    let options = BlitOptions::new_position(
                        x + (tiles.width * sub_rect_view.width()) as i32,
                        y + (tile_y * sub_rect_view.height()) as i32,
                    )
                    .with_sub_rect(sub_rect_view.as_sub_rect())
                    .with_area((remainder.width, sub_rect_view.height()));

                    self.blit_slice(dst, dst_size, &options);
                }

                if remainder.height > 0 {
                    // Render the single leftover remainder
                    let options = BlitOptions::new_position(
                        x + (tiles.width * sub_rect_view.width()) as i32,
                        y + (tiles.height * sub_rect_view.height()) as i32,
                    )
                    .with_sub_rect(sub_rect_view.as_sub_rect())
                    .with_area(remainder);

                    self.blit_slice(dst, dst_size, &options);
                }
            }
        }
    }
        */

    /// Blit a horizontal strip.
    fn blit_horizontal(
        &self,
        target: &mut [Color],
        target_index: Range<usize>,
        source_index: Range<usize>,
    ) {
        // Same size iterators over both our buffer and the output buffer
        let source_iter = self.data[source_index].iter();
        let target_iter = target[target_index].iter_mut();

        // Blit each pixel
        target_iter
            .zip(source_iter)
            .for_each(|(target_pixel, source_pixel)| {
                *target_pixel = Self::blit_pixel(*target_pixel, *source_pixel);
            });
    }

    /// Blit a single pixel.
    ///
    /// The main logic of calculating the resulting color that needs to be drawn.
    #[inline(always)]
    fn blit_pixel(target_pixel: Color, source_pixel: Color) -> Color {
        // Set the pixel from the blit image if the mask value is set
        if (source_pixel >> 24) > 0 {
            // Pixel from the blit buffer is not masked, use it
            source_pixel
        } else {
            // Pixel from the blit buffer is masked, use the original color
            target_pixel
        }
    }
}

impl Blit for BlitBuffer {
    fn size(&self) -> Size {
        self.size
    }

    fn blit_impl(
        &self,
        target: &mut [Color],
        target_width: usize,
        x: usize,
        y: usize,
        u: usize,
        v: usize,
        width: usize,
        height: usize,
    ) {
        let source_width = self.size().width as usize;
        for i in 0..height {
            let x_target = (y + i) * target_width + x;
            let x_source = (v + i) * source_width + u;
            self.blit_horizontal(
                target,
                x_target..(x_target + width),
                x_source..(x_source + width),
            )
        }
    }

    /*
    fn blit(&self, dst: &mut [u32], dst_size: Size, options: &BlitOptions) {
        // Get the total area we need to draw the slices in
        let area = options.area(self.size);

        // Which slices do we need to draw if any
        let slice_projections = self.slice_projections(options, area);

        if slice_projections.is_empty() {
            // Render without projections
            self.blit_slice(dst, dst_size, options);
        } else {
            // Loop over each slice
            slice_projections.into_iter().for_each(|(source, target)| {
                let mut slice_options = options
                    .clone()
                    // Move the position to which part of the slice we need to draw
                    .with_position(options.x + target.x, options.y + target.y)
                    .with_area(target.size);

                // Move the already existing subrectangle if applicable
                slice_options.set_sub_rect(if let Some(sub_rect) = options.sub_rect {
                    sub_rect.shift(source.x, source.y)
                } else {
                    source
                });

                self.blit_slice(dst, dst_size, &slice_options)
            });
        }
    }
    */
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
            2,
            127,
        );
        blit.blit(&mut buffer, Size::new(2, 3)).draw();

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
