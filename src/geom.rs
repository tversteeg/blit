use std::ops::{Add, Div, Mul, Sub};

use num_traits::ToPrimitive;

/// Helper struct for defining sizes.
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

    /// Calculate the size from the length of a buffer and the width.
    pub(crate) fn from_len(len: usize, width: usize) -> Self {
        Self::new(width as u32, (len / width) as u32)
    }

    /// Set the size to the `min()` of another size.
    pub(crate) fn min(&self, other: Self) -> Self {
        Self::new(self.width.min(other.width), self.height.min(other.height))
    }

    /// Tuple of `(width, height)`.
    pub const fn as_tuple(&self) -> (u32, u32) {
        (self.width, self.height)
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

/// Helper struct for defining sub rectangles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubRect {
    /// X offset in pixels.
    pub x: u32,
    /// Y offset in pixels.
    pub y: u32,
    /// Size of the rectangle in pixels.
    pub size: Size,
}

impl SubRect {
    /// Create a new sub-rectangle.
    pub fn new<X, Y>(x: X, y: Y, size: Size) -> Self
    where
        X: ToPrimitive,
        Y: ToPrimitive,
    {
        let x = x.to_u32().unwrap_or_default();
        let y = y.to_u32().unwrap_or_default();

        Self { x, y, size }
    }

    /// Construct from a size with zero coordinates.
    pub const fn from_size(size: Size) -> Self {
        Self { x: 0, y: 0, size }
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
    pub fn right(&self) -> u32 {
        self.x + self.width()
    }

    /// Bottom position, `y + height`.
    pub fn bottom(&self) -> u32 {
        self.y + self.height()
    }

    /// `(x, y, width, height)` slice.
    pub fn as_slice(&self) -> (u32, u32, u32, u32) {
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
        let x = x.to_u32().unwrap_or(0);
        let y = y.to_u32().unwrap_or(0);
        let size = Size::from((width, height));

        Self { x, y, size }
    }
}
