use std::ops::{Add, Sub};

use aseprite::SpritesheetData;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    Blit, BlitBuffer,
};

/// `BlitBuffer` with extra information for rendering as a scalable slice 9 graphic.
///
/// The slice information for scaling it should be exported in the output JSON from aseprite.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Slice9BlitBuffer {
    /// Full image.
    buffer: BlitBuffer,
    /// X position of both vertical slices.
    vertical_slices: (i32, i32),
    /// Y position of both horizontal slices.
    horizontal_slices: (i32, i32),
}

impl Slice9BlitBuffer {
    /// Construct a new buffer for animating a spritesheet.
    pub fn new(buffer: BlitBuffer, info: SpritesheetData) -> Result<Self> {
        // Get the center info from the slices
        let slice = info.meta.slices.get(0).ok_or(Error::NoSlicesInMetadata)?;
        let key = slice
            .keys
            .get(0)
            .ok_or_else(|| Error::NoSliceKeyInSlice(slice.name.clone()))?;
        let center = key
            .center
            .ok_or_else(|| Error::NoSliceCenterInSliceKey(slice.name.clone()))?;

        let vertical_slices = (center.x as i32, (center.x + center.w) as i32);
        let horizontal_slices = (center.y as i32, (center.y + center.h) as i32);

        Ok(Self {
            buffer,
            vertical_slices,
            horizontal_slices,
        })
    }
}

impl Blit for Slice9BlitBuffer {
    fn blit_area(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        (offset_x, offset_y, width, height): (i32, i32, i32, i32),
    ) {
        // If the requested size is smaller or equal to our buffer size just render the whole buffer
        if width <= self.buffer.width() && height <= self.buffer.height() {
            self.buffer.blit(dst, dst_width, (offset_x, offset_y));

            return;
        }

        let dst_height = dst.len() / dst_width;

        // The size can't be smaller than the source or bigger than the target
        let size = Vec2::new(
            width.max(self.buffer.width()).clamp(0, dst_width as i32),
            height.max(self.buffer.height()).clamp(0, dst_height as i32),
        );

        let top_left = Vec2::new(offset_x, offset_y);
        let top_left_inset = Vec2::new(
            top_left.x + self.vertical_slices.0,
            top_left.y + self.horizontal_slices.0,
        );

        let bottom_right = Vec2::new(offset_x + size.x, offset_y + size.y);
        let bottom_right_size = Vec2::new(
            self.buffer.width() - self.vertical_slices.1,
            self.buffer.height() - self.horizontal_slices.1,
        );
        let bottom_right_inset = Vec2::new(
            bottom_right.x - bottom_right_size.x,
            bottom_right.y - bottom_right_size.y,
        );

        // Top left corner
        self.buffer.blit_subrect(
            dst,
            dst_width,
            top_left.as_tuple(),
            (0, 0, self.vertical_slices.0, self.horizontal_slices.0),
        );

        // Top right corner
        self.buffer.blit_subrect(
            dst,
            dst_width,
            (bottom_right_inset.x, top_left.y),
            (
                self.vertical_slices.1,
                0,
                bottom_right_size.x,
                self.horizontal_slices.0,
            ),
        );

        // Bottom left corner
        self.buffer.blit_subrect(
            dst,
            dst_width,
            (top_left.x, bottom_right_inset.y),
            (
                0,
                self.horizontal_slices.1,
                self.vertical_slices.0,
                bottom_right_size.y,
            ),
        );

        // Bottom right corner
        self.buffer.blit_subrect(
            dst,
            dst_width,
            bottom_right_inset.as_tuple(),
            (
                self.vertical_slices.1,
                self.horizontal_slices.1,
                bottom_right_size.x,
                bottom_right_size.y,
            ),
        );

        // Area in the center we need to draw
        let inset = bottom_right_inset - top_left_inset;

        let middle_size = Vec2::new(
            self.vertical_slices.1 - self.vertical_slices.0,
            self.horizontal_slices.1 - self.horizontal_slices.0,
        );

        // Blit top row
        self.buffer.blit_area_subrect(
            dst,
            dst_width,
            (
                top_left_inset.x,
                top_left.y,
                inset.x,
                self.horizontal_slices.0,
            ),
            (
                self.vertical_slices.0,
                0,
                middle_size.x,
                self.horizontal_slices.0,
            ),
        );

        // Blit left column
        self.buffer.blit_area_subrect(
            dst,
            dst_width,
            (
                top_left.x,
                top_left_inset.y,
                self.vertical_slices.0,
                inset.y,
            ),
            (
                0,
                self.horizontal_slices.0,
                self.vertical_slices.0,
                middle_size.y,
            ),
        );

        // Blit bottom row
        self.buffer.blit_area_subrect(
            dst,
            dst_width,
            (
                top_left_inset.x,
                bottom_right_inset.y,
                inset.x,
                bottom_right_size.y,
            ),
            (
                self.vertical_slices.0,
                self.horizontal_slices.1,
                middle_size.x,
                bottom_right_size.y,
            ),
        );

        // Blit right column
        self.buffer.blit_area_subrect(
            dst,
            dst_width,
            (
                bottom_right_inset.x,
                top_left_inset.y,
                bottom_right_size.x,
                inset.y,
            ),
            (
                self.vertical_slices.1,
                self.horizontal_slices.0,
                bottom_right_size.x,
                middle_size.y,
            ),
        );

        // Blit center
        self.buffer.blit_area_subrect(
            dst,
            dst_width,
            (top_left_inset.x, top_left_inset.y, inset.x, inset.y),
            (
                self.vertical_slices.0,
                self.horizontal_slices.0,
                middle_size.x,
                middle_size.y,
            ),
        )
    }

    fn blit_area_subrect(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        (offset_x, offset_y, width, height): (i32, i32, i32, i32),
        sub_rect: (i32, i32, i32, i32),
    ) {
        // If the requested size is smaller or equal to our buffer size just render the whole buffer
        if width <= self.buffer.width() && height <= self.buffer.height() {
            self.buffer.blit(dst, dst_width, (offset_x, offset_y));

            return;
        }

        let dst_height = dst.len() / dst_width;

        // The size can't be smaller than the source or bigger than the target
        let size = Vec2::new(
            width.max(self.buffer.width()).clamp(0, dst_width as i32),
            height.max(self.buffer.height()).clamp(0, dst_height as i32),
        );

        let top_left = Vec2::new(offset_x, offset_y);
        let top_left_inset = Vec2::new(
            top_left.x + self.vertical_slices.0,
            top_left.y + self.horizontal_slices.0,
        );

        let bottom_right = Vec2::new(offset_x + size.x, offset_y + size.y);
        let bottom_right_size = Vec2::new(
            self.buffer.width() - self.vertical_slices.1,
            self.buffer.height() - self.horizontal_slices.1,
        );
        let bottom_right_inset = Vec2::new(
            bottom_right.x - bottom_right_size.x,
            bottom_right.y - bottom_right_size.y,
        );

        // Top left corner
        self.buffer.blit_subrect(
            dst,
            dst_width,
            top_left.as_tuple(),
            (0, 0, self.vertical_slices.0, self.horizontal_slices.0),
        );

        // Top right corner
        self.buffer.blit_subrect(
            dst,
            dst_width,
            (bottom_right_inset.x, top_left.y),
            (
                self.vertical_slices.1,
                0,
                bottom_right_size.x,
                self.horizontal_slices.0,
            ),
        );

        // Bottom left corner
        self.buffer.blit_subrect(
            dst,
            dst_width,
            (top_left.x, bottom_right_inset.y),
            (
                0,
                self.horizontal_slices.1,
                self.vertical_slices.0,
                bottom_right_size.y,
            ),
        );

        // Bottom right corner
        self.buffer.blit_subrect(
            dst,
            dst_width,
            bottom_right_inset.as_tuple(),
            (
                self.vertical_slices.1,
                self.horizontal_slices.1,
                bottom_right_size.x,
                bottom_right_size.y,
            ),
        );

        // Area in the center we need to draw
        let inset = bottom_right_inset - top_left_inset;

        let middle_size = Vec2::new(
            self.vertical_slices.1 - self.vertical_slices.0,
            self.horizontal_slices.1 - self.horizontal_slices.0,
        );

        // Amount of sections needed for drawing the middle and edges
        let sections = Vec2::new(
            div_up(inset.x, middle_size.x),
            div_up(inset.y, middle_size.y),
        );

        // Position where the remainders should start on the bottom right
        let remainder = Vec2::new(
            top_left_inset.x + (sections.x - 1) * middle_size.x,
            top_left_inset.y + (sections.y - 1) * middle_size.y,
        );
        // Size of the remaining pieces
        let remainder_size = Vec2::new(
            bottom_right_inset.x - remainder.x,
            bottom_right_inset.y - remainder.y,
        );

        // Blit top row remainder
        self.buffer.blit_subrect(
            dst,
            dst_width,
            (remainder.x, top_left.y),
            (
                self.vertical_slices.0,
                0,
                remainder_size.x,
                self.horizontal_slices.0,
            ),
        );

        // Blit bottom row remainder
        self.buffer.blit_subrect(
            dst,
            dst_width,
            (remainder.x, bottom_right_inset.y),
            (
                self.vertical_slices.0,
                self.horizontal_slices.1,
                remainder_size.x,
                bottom_right_size.y,
            ),
        );

        // Blit left row remainder
        self.buffer.blit_subrect(
            dst,
            dst_width,
            (top_left.x, remainder.y),
            (
                0,
                self.horizontal_slices.0,
                self.vertical_slices.0,
                remainder_size.y,
            ),
        );

        // Blit right row remainder
        self.buffer.blit_subrect(
            dst,
            dst_width,
            (bottom_right_inset.x, remainder.y),
            (
                self.vertical_slices.1,
                self.horizontal_slices.0,
                bottom_right_size.x,
                remainder_size.y,
            ),
        );

        // Blit center remainder
        self.buffer.blit_subrect(
            dst,
            dst_width,
            (remainder.x, remainder.y),
            (
                self.vertical_slices.0,
                self.horizontal_slices.0,
                remainder_size.x,
                remainder_size.y,
            ),
        );

        // Blit center and the top and bottom rows
        for section_x in 0..(sections.x - 1) {
            let x = top_left_inset.x + section_x * middle_size.x;

            // Blit center
            for section_y in 0..(sections.y - 1) {
                let y = top_left_inset.y + section_y * middle_size.y;

                self.buffer.blit_subrect(
                    dst,
                    dst_width,
                    (x, y),
                    (
                        self.vertical_slices.0,
                        self.horizontal_slices.0,
                        middle_size.x,
                        middle_size.y,
                    ),
                );
            }

            // Blit top row
            self.buffer.blit_subrect(
                dst,
                dst_width,
                (x, top_left.y),
                (
                    self.vertical_slices.0,
                    0,
                    middle_size.x,
                    self.horizontal_slices.0,
                ),
            );

            // Blit bottom row
            self.buffer.blit_subrect(
                dst,
                dst_width,
                (x, bottom_right_inset.y),
                (
                    self.vertical_slices.0,
                    self.horizontal_slices.1,
                    middle_size.x,
                    bottom_right_size.y,
                ),
            );

            // Blit center horizontal remainder
            self.buffer.blit_subrect(
                dst,
                dst_width,
                (x, remainder.y),
                (
                    self.vertical_slices.0,
                    self.horizontal_slices.1,
                    middle_size.x,
                    remainder_size.y,
                ),
            );
        }

        // Blit left and right rows
        for section_y in 0..(sections.y - 1) {
            let y = top_left_inset.y + section_y * middle_size.y;

            // Blit left row
            self.buffer.blit_subrect(
                dst,
                dst_width,
                (top_left.x, y),
                (
                    0,
                    self.horizontal_slices.0,
                    self.vertical_slices.0,
                    middle_size.y,
                ),
            );

            // Blit right row
            self.buffer.blit_subrect(
                dst,
                dst_width,
                (bottom_right_inset.x, y),
                (
                    self.vertical_slices.1,
                    self.horizontal_slices.0,
                    bottom_right_size.x,
                    middle_size.y,
                ),
            );

            // Blit center vertical remainder
            self.buffer.blit_subrect(
                dst,
                dst_width,
                (remainder.x, y),
                (
                    self.vertical_slices.1,
                    self.horizontal_slices.0,
                    remainder_size.x,
                    middle_size.y,
                ),
            );
        }
    }

    fn size(&self) -> (i32, i32) {
        self.buffer.size()
    }
}

// Simple helper struct to make calculating XY positions a lot easier.
#[derive(Clone, Copy)]
struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Vec2 {
    /// Create the vector.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert to tuple.
    pub fn as_tuple(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// Divide integer by rounding up.
///
/// Source: https://www.reddit.com/r/rust/comments/bk7v15/my_next_favourite_way_to_divide_integers_rounding/
fn div_up(a: i32, b: i32) -> i32 {
    (a + (b - 1)) / b
}
