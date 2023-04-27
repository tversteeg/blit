//! Helper structs for simple geometric calculations.

use std::ops::{Add, Div, Mul, Rem, Sub};

use num_traits::ToPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Helper struct for defining sizes.
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

/// Helper struct for defining sub rectangles.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubRect {
    /// X offset in pixels.
    pub x: i32,
    /// Y offset in pixels.
    pub y: i32,
    /// Size of the rectangle in pixels.
    pub size: Size,
}

impl SubRect {
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
}

impl<X, Y, W, H> From<(X, Y, W, H)> for SubRect
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
