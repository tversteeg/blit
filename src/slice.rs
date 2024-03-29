//! Divide the source buffer into multiple sections and repeat the chosen section to fill the area.
//!
//! # Example
//!
//! ```rust
//! use blit::{BlitOptions, slice::Slice};
//!
//! // Create a slice 9 type split of a 9x9 image in 3 exact parts
//! BlitOptions {
//!    vertical_slice: Some(Slice::ternary(3, 6)),
//!    horizontal_slice: Some(Slice::ternary(3, 6)),
//!    ..Default::default()
//! };
//!
//! // Although you probably want, note that the right and bottom coordinates are not absolute anymore, here they are the width and height of the center rectangle
//! BlitOptions::new().with_slice9((3, 3, 3, 3));
//! ```

use num_traits::ToPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{Size, SubRect};

/// Divide the source buffer into multiple sections and repeat the chosen section to fill the area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Slice {
    /// Split the buffer into two and repeat one of the sections.
    Binary {
        /// Position between the first and last section to split.
        split: u32,
        /// Which of the sections to scale when the area is bigger than the total size.
        repeat: BinarySection,
    },

    /// Split the buffer into three and repeat the middle section.
    ///
    /// If you want to resize either the left or right part a binary section is used.
    Ternary {
        /// Position between the first and the middle section.
        split_first: u32,
        /// Position between the middle and last section.
        split_last: u32,
    },
}

impl Slice {
    /// Create a binary split where the first section is scaled.
    ///
    /// When horizontal this is the top section.
    /// When vertical this is the left section.
    pub fn binary_first<S>(split: S) -> Self
    where
        S: ToPrimitive,
    {
        let split = split.to_u32().unwrap_or_default();

        Self::Binary {
            split,
            repeat: BinarySection::First,
        }
    }

    /// Create a binary split where the last section is scaled.
    ///
    /// When horizontal this is the bottom section.
    /// When vertical this is the right section.
    pub fn binary_last<S>(split: S) -> Self
    where
        S: ToPrimitive,
    {
        let split = split.to_u32().unwrap_or_default();

        Self::Binary {
            split,
            repeat: BinarySection::Last,
        }
    }

    /// Create a ternary split where the middle section is scaled.
    ///
    /// With both horizontal and vertical this is the middle section.
    pub fn ternary<S1, S2>(split_first: S1, split_last: S2) -> Self
    where
        S1: ToPrimitive,
        S2: ToPrimitive,
    {
        let split_first = split_first.to_u32().unwrap_or_default();
        let split_last = split_last.to_u32().unwrap_or_default();

        Self::Ternary {
            split_first,
            split_last,
        }
    }

    /// Divide the given single dimensional area by the slice ranges.
    pub(crate) fn divide_area_iter(
        &self,
        source_length: u32,
        target_length: u32,
    ) -> impl Iterator<Item = SliceProjection> {
        match self {
            Slice::Binary { split, repeat } => {
                // Find the middle intersection depending on which part needs to scale
                let middle = match repeat {
                    BinarySection::First => target_length.saturating_sub(*split),
                    BinarySection::Last => *split,
                };

                [
                    (0, middle, 0, *split),
                    (middle, target_length, *split, source_length),
                    // The (0, 0) pair will be removed by the filter, we have to add this otherwise the compiler will complain about the iterators not being the same size
                    (0, 0, 0, 0),
                ]
                .into_iter()
            }
            Slice::Ternary {
                split_first,
                split_last,
            } => {
                // Find the two middle intersections depending on which part needs to scale
                let (middle_first, middle_second) = (
                    *split_first,
                    target_length.saturating_sub(source_length.saturating_sub(*split_last)),
                );

                // Ensure they don't go out of bounds
                let middle_first = middle_first.min(middle_second);
                let middle_second = middle_second.clamp(middle_first, target_length);

                [
                    (0, middle_first, 0, *split_first),
                    (middle_first, middle_second, *split_first, *split_last),
                    (middle_second, target_length, *split_last, source_length),
                ]
                .into_iter()
            }
        }
        // Remove empty ranges
        .filter(|(target_start, target_end, source_start, source_end)| {
            target_start < target_end && source_start < source_end
        })
        .map(|(target_start, target_end, source_start, source_end)| {
            SliceProjection::new(source_start, source_end, target_start, target_end)
        })
    }
}

/// Choose which split of the binary section to scale in a repeating fashion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BinarySection {
    /// Repeat the first section.
    ///
    /// When horizontal this is the top section.
    /// When vertical this is the left section.
    First,
    /// Repeat the second section.
    ///
    /// When horizontal this is the bottom section.
    /// When vertical this is the right section.
    Last,
}

/// How a slice must be rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SliceProjection {
    /// Left part of the absolute range in the source buffer to take the image from.
    pub source_start: u32,
    /// Right part of the absolute range in the source buffer to take the image from.
    pub source_end: u32,
    /// Left part of the relative range in the destination buffer to draw into.
    pub target_start: u32,
    /// Right part of the relative range in the destination buffer to draw into.
    pub target_end: u32,
}

impl SliceProjection {
    /// Construct a new projection.
    pub fn new(source_start: u32, source_end: u32, target_start: u32, target_end: u32) -> Self {
        Self {
            source_start,
            source_end,
            target_start,
            target_end,
        }
    }

    /// Amount of pixels of the range.
    pub fn source_amount(&self) -> u32 {
        self.source_end - self.source_start
    }

    /// Amount of pixels of the range.
    pub fn target_amount(&self) -> u32 {
        self.target_end - self.target_start
    }

    /// Create a `(source, target)` rectangle tuple with a static Y axis.
    pub fn into_sub_rects_static_y(self, y_size: u32) -> (SubRect, SubRect) {
        let source = SubRect::from((self.source_start, 0, self.source_amount(), y_size));
        let target = SubRect::from((self.target_start, 0, self.target_amount(), y_size));

        (source, target)
    }

    /// Create a `(source, target)` rectangle tuple with a static X axis.
    pub fn into_sub_rects_static_x(self, x_size: u32) -> (SubRect, SubRect) {
        let source = SubRect::from((0, self.source_start, x_size, self.source_amount()));
        let target = SubRect::from((0, self.target_start, x_size, self.target_amount()));

        (source, target)
    }

    /// Create a `(source, target)` rectangle tuple from horizontal and vertical projections.
    pub fn combine_into_sub_rects(
        horizontal: &SliceProjection,
        vertical: &SliceProjection,
    ) -> (SubRect, SubRect) {
        let source = SubRect::new(
            horizontal.source_start,
            vertical.source_start,
            Size::new(horizontal.source_amount(), vertical.source_amount()),
        );

        let target = SubRect::new(
            horizontal.target_start,
            vertical.target_start,
            Size::new(horizontal.target_amount(), vertical.target_amount()),
        );

        (source, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice9() {
        let (horizontal_slice, vertical_slice) = (Slice::ternary(10, 20), Slice::ternary(25, 50));

        let horizontal_projs = horizontal_slice
            .divide_area_iter(30, 100)
            .collect::<Vec<_>>();
        let vertical_projs = vertical_slice.divide_area_iter(75, 150).collect::<Vec<_>>();
        assert_eq!(
            horizontal_projs,
            [
                SliceProjection::new(0, 10, 0, 10),
                SliceProjection::new(10, 20, 10, 90),
                SliceProjection::new(20, 30, 90, 100)
            ]
        );

        assert_eq!(
            vertical_projs,
            [
                SliceProjection::new(0, 25, 0, 25),
                SliceProjection::new(25, 50, 25, 125),
                SliceProjection::new(50, 75, 125, 150)
            ]
        );

        // Test mapping to subrectangles
        assert_eq!(
            SliceProjection::combine_into_sub_rects(&horizontal_projs[0], &vertical_projs[0]),
            ((0, 0, 10, 25).into(), (0, 0, 10, 25).into())
        );
        assert_eq!(
            SliceProjection::combine_into_sub_rects(&horizontal_projs[1], &vertical_projs[1]),
            ((10, 25, 10, 25).into(), (10, 25, 80, 100).into())
        );
        assert_eq!(
            SliceProjection::combine_into_sub_rects(&horizontal_projs[2], &vertical_projs[2]),
            ((20, 50, 10, 25).into(), (90, 125, 10, 25).into())
        );

        assert_eq!(
            horizontal_projs[0].clone().into_sub_rects_static_y(25),
            ((0, 0, 10, 25).into(), (0, 0, 10, 25).into())
        );
        assert_eq!(
            horizontal_projs[1].clone().into_sub_rects_static_y(25),
            ((10, 0, 10, 25).into(), (10, 0, 80, 25).into())
        );
        assert_eq!(
            horizontal_projs[2].clone().into_sub_rects_static_y(25),
            ((20, 0, 10, 25).into(), (90, 0, 10, 25).into())
        );

        assert_eq!(
            vertical_projs[0].clone().into_sub_rects_static_x(10),
            ((0, 0, 10, 25).into(), (0, 0, 10, 25).into())
        );
        assert_eq!(
            vertical_projs[1].clone().into_sub_rects_static_x(10),
            ((0, 25, 10, 25).into(), (0, 25, 10, 100).into())
        );
        assert_eq!(
            vertical_projs[2].clone().into_sub_rects_static_x(10),
            ((0, 50, 10, 25).into(), (0, 125, 10, 25).into())
        );
    }
}
