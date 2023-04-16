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
//! }
//!
//! // Although you probably want
//! BlitOptions::slice9((x, y), 3, 6, 3, 6);
//! ```

/// Divide the source buffer into multiple sections and repeat the chosen section to fill the area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slice {
    /// Split the buffer into two and repeat one of the sections.
    Binary {
        /// Position between the first and last section to split.
        split: i32,
        /// Which of the sections to scale when the area is bigger than the total size.
        repeat: BinarySection,
    },

    /// Split the buffer into three and repeat one of the sections.
    Ternary {
        /// Position between the first and the middle section.
        split_first: i32,
        /// Position between the middle and last section.
        split_last: i32,
        /// Which of the sections to scale when the area is bigger than the total size.
        repeat: TernarySection,
    },
}

impl Slice {
    /// Create a binary split where the first section is chosen.
    ///
    /// When horizontal this is the top section.
    /// When vertical this is the left section.
    pub fn binary_first(split: i32) -> Self {
        Self::Binary {
            split,
            repeat: BinarySection::First,
        }
    }

    /// Create a binary split where the last section is chosen.
    ///
    /// When horizontal this is the bottom section.
    /// When vertical this is the right section.
    pub fn binary_last(split: i32) -> Self {
        Self::Binary {
            split,
            repeat: BinarySection::Last,
        }
    }

    /// Create a ternary split where the first section is chosen.
    ///
    /// When horizontal this is the top section.
    /// When vertical this is the left section.
    pub fn ternary_first(split_first: i32, split_last: i32) -> Self {
        Self::Ternary {
            split_first,
            split_last,
            repeat: TernarySection::First,
        }
    }

    /// Create a ternary split where the last section is chosen.
    ///
    /// With both horizontal and vertical this is the middle section.
    pub fn ternary_middle(split_first: i32, split_last: i32) -> Self {
        Self::Ternary {
            split_first,
            split_last,
            repeat: TernarySection::Middle,
        }
    }

    /// Create a ternary split where the last section is chosen.
    ///
    /// When horizontal this is the bottom section.
    /// When vertical this is the right section.
    pub fn ternary_last(split_first: i32, split_last: i32) -> Self {
        Self::Ternary {
            split_first,
            split_last,
            repeat: TernarySection::Last,
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

/// Choose which split of the ternary section to scale in a repeating fashion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TernarySection {
    /// Repeat the first section.
    ///
    /// When horizontal this is the top section.
    /// When vertical this is the left section.
    First,
    /// Repeat the middle section.
    ///
    /// With both horizontal and vertical this is the middle section.
    Middle,
    /// Repeat the second section.
    ///
    /// When horizontal this is the bottom section.
    /// When vertical this is the right section.
    Last,
}
