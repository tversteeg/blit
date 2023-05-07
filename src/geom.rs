//! Helper structs for simple geometric calculations.

use std::ops::{Add, Div, Mul, Range, Rem, Sub};

use num_traits::ToPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Coordinates, used for UV and offsets.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate {
    /// X position in pixels.
    pub x: i32,
    /// Y position in pixels.
    pub y: i32,
}

impl Coordinate {
    /// Create a new size.
    pub fn new<X, Y>(x: X, y: Y) -> Self
    where
        X: ToPrimitive,
        Y: ToPrimitive,
    {
        let x = x.to_i32().unwrap_or_default();
        let y = y.to_i32().unwrap_or_default();

        Self { x, y }
    }

    /// Create from a `i32` tuple.
    pub const fn from_tuple((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }

    /// Tuple of `(x, y)`.
    pub const fn as_tuple(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl<X, Y> From<(X, Y)> for Coordinate
where
    X: ToPrimitive,
    Y: ToPrimitive,
{
    fn from((x, y): (X, Y)) -> Self {
        let x = x.to_i32().unwrap_or_default();
        let y = y.to_i32().unwrap_or_default();

        Self { x, y }
    }
}

impl Rem<Coordinate> for Coordinate {
    type Output = Self;

    fn rem(self, rhs: Coordinate) -> Self::Output {
        Self {
            x: self.x % rhs.x,
            y: self.y % rhs.y,
        }
    }
}

/// Sizes.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Size {
    /// Create a new size.
    pub fn new<W, H>(width: W, height: H) -> Self
    where
        W: ToPrimitive,
        H: ToPrimitive,
    {
        let width = width.to_u32().unwrap_or_default();
        let height = height.to_u32().unwrap_or_default();

        Self { width, height }
    }

    /// Create from a `u32` tuple.
    pub const fn from_tuple((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }

    /// Tuple of `(width, height)`.
    pub const fn as_tuple(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// `(width, height)` -> `(x, y)`.
    pub const fn as_coordinate(&self) -> Coordinate {
        Coordinate {
            x: self.width as i32,
            y: self.height as i32,
        }
    }

    /// Amount of pixels.
    pub const fn pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Calculate the size from the length of a buffer and the width.
    pub(crate) const fn from_len(len: usize, width: usize) -> Self {
        Self {
            width: width as u32,
            height: (len / width) as u32,
        }
    }

    /// Set the size to the `min()` of another size.
    pub(crate) fn min(&self, other: Self) -> Self {
        Self {
            width: self.width.min(other.width),
            height: self.height.min(other.height),
        }
    }
}

impl<W, H> From<(W, H)> for Size
where
    W: ToPrimitive,
    H: ToPrimitive,
{
    fn from((width, height): (W, H)) -> Self {
        let width = width.to_u32().unwrap_or_default();
        let height = height.to_u32().unwrap_or_default();

        Self { width, height }
    }
}

impl Add<Size> for Size {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl Sub<Size> for Size {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl Mul<Size> for Size {
    type Output = Self;

    fn mul(self, rhs: Size) -> Self::Output {
        Self {
            width: self.width * rhs.width,
            height: self.height * rhs.height,
        }
    }
}

impl<T: ToPrimitive> Mul<T> for Size {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        // Don't use 0 as a default, a 1 does nothing
        let rhs = rhs.to_u32().unwrap_or(1);

        Self {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl Div<Size> for Size {
    type Output = Self;

    fn div(self, rhs: Size) -> Self::Output {
        Self {
            width: self.width / rhs.width,
            height: self.height / rhs.height,
        }
    }
}

impl<T: ToPrimitive> Div<T> for Size {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        // Don't use 0 as a default otherwise we get a division by zero
        let rhs = rhs.to_u32().unwrap_or(1);

        Self {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

impl Rem<Size> for Size {
    type Output = Self;

    fn rem(self, rhs: Size) -> Self::Output {
        Self {
            width: self.width % rhs.width,
            height: self.height % rhs.height,
        }
    }
}

impl<T: ToPrimitive> Rem<T> for Size {
    type Output = Self;

    fn rem(self, rhs: T) -> Self::Output {
        // Don't use 0 as a default otherwise we get a division by zero
        let rhs = rhs.to_u32().unwrap_or_default();

        Self {
            width: self.width % rhs,
            height: self.height % rhs,
        }
    }
}

/// Helper struct for defining sub-rectangles.
///
/// A sub-rectangle is a rectangle that's part of a bigger rectangle.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// X offset in pixels.
    pub x: i32,
    /// Y offset in pixels.
    pub y: i32,
    /// Size of the rectangle in pixels.
    pub size: Size,
}

impl Rect {
    /// Zero sized.
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        size: Size {
            width: 0,
            height: 0,
        },
    };

    /// Create a new sub-rectangle.
    pub fn new<X, Y, S>(x: X, y: Y, size: S) -> Self
    where
        X: ToPrimitive,
        Y: ToPrimitive,
        S: Into<Size>,
    {
        let x = x.to_i32().unwrap_or_default();
        let y = y.to_i32().unwrap_or_default();
        let size = size.into();

        Self { x, y, size }
    }

    /// Construct from a size with zero coordinates.
    pub fn from_size<S>(size: S) -> Self
    where
        S: Into<Size>,
    {
        let (x, y) = (0, 0);
        let size = size.into();

        Self { x, y, size }
    }

    /// Shift the left and top position while keeping the right and bottom position at the same spot.
    pub fn shift<X, Y>(&self, new_x: X, new_y: Y) -> Self
    where
        X: ToPrimitive,
        Y: ToPrimitive,
    {
        let new_x = new_x.to_i32().unwrap_or_default();
        let new_y = new_y.to_i32().unwrap_or_default();

        let (right, bottom) = (self.right(), self.bottom());
        let (x, y) = (new_x.clamp(0, right), new_y.clamp(0, bottom));

        let size = Size::new((right - self.x).max(0), (bottom - self.y).max(0));

        Self { x, y, size }
    }

    /// Width as `u32`.
    pub fn width(&self) -> u32 {
        self.size.width
    }

    /// Height as `u32`.
    pub fn height(&self) -> u32 {
        self.size.height
    }

    /// Right position, `x + width`.
    pub fn right(&self) -> i32 {
        self.x + self.width() as i32
    }

    /// Bottom position, `y + height`.
    pub fn bottom(&self) -> i32 {
        self.y + self.height() as i32
    }

    /// `(x, y, width, height)` slice.
    pub fn as_slice(&self) -> (i32, i32, u32, u32) {
        (self.x, self.y, self.size.width, self.size.height)
    }

    /// Iterator over horizontal ranges in the buffer the rect is based on.
    ///
    /// Each range represents a slice of bytes that can be taken.
    /// Bounds checks should have already been done by the new function.
    pub fn parent_ranges_iter<S>(&self, parent_size: S) -> impl Iterator<Item = Range<usize>>
    where
        S: Into<Size>,
    {
        let parent_size = parent_size.into();

        let (width, height) = (self.width() as usize, self.height() as usize);
        let (start_x, start_y) = (self.x.max(0) as usize, self.y.max(0) as usize);
        let end_y = start_y + height;

        let parent_width = parent_size.width as usize;

        (start_y..end_y).map(move |y| {
            let start_x = y * parent_width + start_x;
            let end_x = start_x + width;

            start_x..end_x
        })
    }
}

impl<X, Y, W, H> From<(X, Y, W, H)> for Rect
where
    X: ToPrimitive,
    Y: ToPrimitive,
    W: ToPrimitive,
    H: ToPrimitive,
{
    fn from((x, y, width, height): (X, Y, W, H)) -> Self {
        let x = x.to_i32().unwrap_or(0);
        let y = y.to_i32().unwrap_or(0);
        let size = Size::from((width, height));

        Self { x, y, size }
    }
}

impl Add<Coordinate> for Rect {
    type Output = Self;

    fn add(mut self, rhs: Coordinate) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;

        self
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_shift() {
        let rect = Rect::new(10, 10, (10, 10));
        assert_eq!(rect.shift(15, 15), Rect::new(15, 15, (5, 5)));
    }
}
*/
