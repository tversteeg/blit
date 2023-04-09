use std::ops::{Add, Sub};

use aseprite::SpritesheetData;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    BlitBuffer,
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

    /// Draw the current frame using the animation info.
    pub fn blit(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        (offset_x, offset_y, width, height): (i32, i32, i32, i32),
    ) {
        let size = Vec2::new(width, height);

        let top_left = Vec2::new(offset_x, offset_y);
        let top_left_inset = Vec2::new(
            top_left.x + self.vertical_slices.0,
            top_left.y + self.horizontal_slices.0,
        )
        .clamp(size);
        let bottom_right = Vec2::new(offset_x + width, offset_y + height);
        let bottom_right_inset = Vec2::new(
            bottom_right.x - self.vertical_slices.1,
            bottom_right.y - self.horizontal_slices.0,
        )
        .clamp(size);

        let bottom_right_size = Vec2::new(
            self.buffer.width() - self.vertical_slices.1,
            self.buffer.height() - self.horizontal_slices.1,
        );

        // Top left corner
        self.buffer.blit_rect(
            dst,
            dst_width,
            top_left.as_tuple(),
            (0, 0, self.vertical_slices.0, self.horizontal_slices.0),
        );

        // Top right corner
        self.buffer.blit_rect(
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
        self.buffer.blit_rect(
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
        self.buffer.blit_rect(
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

        // Blit center and the top and bottom rows
        for section_x in 0..sections.x {
            let x = self.vertical_slices.0 + section_x * middle_size.x;

            // Blit center
            for section_y in 0..sections.y {
                let y = self.horizontal_slices.0 + section_y * middle_size.y;

                self.buffer.blit_rect(
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
            self.buffer.blit_rect(
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
            self.buffer.blit_rect(
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
        }

        // Blit left and right rows
        for section_y in 0..sections.y {
            let y = self.horizontal_slices.0 + section_y * middle_size.y;

            // Blit left row
            self.buffer.blit_rect(
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
            self.buffer.blit_rect(
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
        }
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

    /// Clamp to 0, 0 and the size passed.
    pub fn clamp(self, bounds: Vec2) -> Self {
        Self {
            x: self.x.min(bounds.x).max(0),
            y: self.y.min(bounds.y).max(0),
        }
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
