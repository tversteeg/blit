//! Divide the source buffer into multiple sections and repeat the chosen section to fill the area.
//!
//! # Example
//!
//! ```rust
//! use blit::{BlitOptions, slice::Slice};
//!
//! # let (x, y) = (0, 0);
//! // Create a slice 9 type split of a 9x9 image in 3 exact parts
//! BlitOptions {
//!    x,
//!    y,
//!    vertical_slices: Some(Slice::ternary_middle(3, 6)),
//!    horizontal_slices: Some(Slice::ternary_middle(3, 6)),
//!    ..Default::default()
//! };
//!
//! // Although you probably want
//! BlitOptions::new_slice9(x, y, 3, 6, 3, 6);
//! ```

use crate::{Size, SubRect};

/// Divide the source buffer into multiple sections and repeat the chosen section to fill the area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Create a binary split where the first section is chosen.
    ///
    /// When horizontal this is the top section.
    /// When vertical this is the left section.
    pub const fn binary_first(split: u32) -> Self {
        Self::Binary {
            split,
            repeat: BinarySection::First,
        }
    }

    /// Create a binary split where the last section is chosen.
    ///
    /// When horizontal this is the bottom section.
    /// When vertical this is the right section.
    pub const fn binary_last(split: u32) -> Self {
        Self::Binary {
            split,
            repeat: BinarySection::Last,
        }
    }

    /// Create a ternary split where the last section is chosen.
    ///
    /// With both horizontal and vertical this is the middle section.
    pub const fn ternary(split_first: u32, split_last: u32) -> Self {
        Self::Ternary {
            split_first,
            split_last,
        }
    }

    /// Divide the given single dimensional area by the slice ranges.
    ///
    /// The tuple returned has a pair of start and end pixels.
    pub fn divide_area(&self, target_length: u32) -> Vec<(u32, u32)> {
        match self {
            Slice::Binary { split, repeat } => {
                // Find the middle intersection depending on which part needs to scale
                let middle = match repeat {
                    BinarySection::First => target_length.saturating_sub(*split),
                    BinarySection::Last => *split,
                }
                .min(target_length);

                vec![(0, middle), (middle, target_length)]
            }
            Slice::Ternary {
                split_first,
                split_last,
            } => {
                // Find the two middle intersections depending on which part needs to scale
                let (middle_first, middle_second) =
                    (*split_first, target_length.saturating_sub(*split_last));

                // Ensure they don't go out of bounds
                let middle_first = middle_first.min(middle_second);
                let middle_second = middle_second.clamp(middle_first, target_length);

                vec![
                    (0, middle_first),
                    (middle_first, middle_second),
                    (middle_second, target_length),
                ]
            }
        }
    }
}

/// Choose which split of the binary section to scale in a repeating fashion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
