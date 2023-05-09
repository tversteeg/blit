use std::ops::Range;

use num_traits::ToPrimitive;

use crate::{prelude::Coordinate, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ImageView(Rect);

impl ImageView {
    /// Create a new view based on the size of the original buffer.
    ///
    /// When it's `None` the view doesn't contain any actual pixels.
    pub fn new<T, P>(target: T, parent: P) -> Option<Self>
    where
        T: Into<Rect>,
        P: Into<Rect>,
    {
        let (target, parent) = (target.into(), parent.into());

        // No view when the size of the target is zero
        if target.width() == 0 || target.height() == 0 {
            return None;
        }

        let (parent_right, parent_bottom) = (parent.right(), parent.bottom());

        // No pixels will be drawn when the view taken is outside of the bounds of the original image
        if target.x <= parent.x - target.width() as i32
            || target.y <= parent.y - target.height() as i32
            || target.x >= parent_right
            || target.y >= parent_bottom
        {
            return None;
        }

        // Clip the target to the left and top side
        let (x, y) = (target.x.max(parent.x), target.y.max(parent.y));

        // How much pixels are subtracted from the left and top side by clipping
        let (subtract_x, subtract_y) = (x - target.x, y - target.y);

        // Clip the width and height to right and bottom side if applicable
        let (actual_width, actual_height) = (
            (target.width() as i32 - subtract_x).min(parent_right - target.x),
            (target.height() as i32 - subtract_y).min(parent_bottom - target.y),
        );

        Some(Self::new_unchecked(
            x,
            y,
            Size::new(actual_width, actual_height),
        ))
    }

    /// Create a new view without checking if it fits in the parent.
    pub fn new_unchecked<X, Y, S>(x: X, y: Y, size: S) -> Self
    where
        X: ToPrimitive,
        Y: ToPrimitive,
        S: Into<Size>,
    {
        Self(Rect::new(x, y, size))
    }

    /// Create a full view of a complete buffer without checking if it fits in the parent.
    pub fn full(size: Size) -> Self {
        Self(Rect { x: 0, y: 0, size })
    }

    /// Iterator over horizontal ranges in the buffer the view is based on.
    ///
    /// Each range represents a slice of bytes that can be taken.
    /// Bounds checks should have already been done by the new function.
    pub fn parent_ranges_iter(&self, parent_size: Size) -> impl Iterator<Item = Range<usize>> {
        self.0.parent_ranges_iter(parent_size)
    }

    /// Size in pixels.
    pub fn size(&self) -> Size {
        self.0.size
    }

    /// Create a sub-view based on this view.
    pub fn sub<S>(&self, target: S) -> Option<Self>
    where
        S: Into<Rect>,
    {
        Self::new(target, self.as_sub_rect())
    }

    /// Create a sub rectangle where the position can be negative.
    pub fn sub_i32(&self, x: i32, y: i32, size: Size) -> Option<Self> {
        // Don't allow the X and Y to be smaller than 0
        let (clip_x, clip_y) = (x.max(0), y.max(0));
        // Calculate how much pixels got clipped
        let (offset_x, offset_y) = (clip_x - x, clip_y - y);

        // Remove the clipped pixels from the size
        let (new_width, new_height) = (
            (size.width as i32 - offset_x).max(0),
            (size.height as i32 - offset_y).max(0),
        );

        Self::new((x, y, new_width, new_height), self.as_sub_rect())
    }

    /// Shift the left and top position while keeping the right and bottom position at the same spot.
    pub fn shift<X, Y>(&self, new_x: X, new_y: Y) -> (Coordinate, Self)
    where
        X: ToPrimitive,
        Y: ToPrimitive,
    {
        let (coord, rect) = self.0.shift(new_x, new_y);

        (coord, Self(rect))
    }

    /// Try to make the size of the rectangle smaller.
    pub fn shrink<S>(&mut self, size: S)
    where
        S: Into<Size>,
    {
        self.0.size = size.into().min(self.size());
    }

    /// Clip the view to fit in another one.
    pub fn clip(&mut self, other: ImageView) {
        let (right, bottom) = (self.0.right(), self.0.bottom());

        // Clip the left edge
        if self.0.x < other.x() {
            self.0.x = other.x();
            let width = (right - self.0.x);
            assert!(width >= 0);
            self.0.size.width = width as u32;
        }
        // Clip the top edge
        if self.0.y < other.y() {
            self.0.y = other.y();
            let height = (bottom - self.0.y);
            assert!(height >= 0);
            self.0.size.height = height as u32;
        }
        // Clip the right edge
        if right > other.0.right() {
            let width = (other.0.right() - self.0.x);
            assert!(width >= 0);
            self.0.size.width = width as u32;
        }
        // Clip the bottom edge
        if bottom > other.0.bottom() {
            let height = (other.0.bottom() - self.0.y);
            assert!(height >= 0);
            self.0.size.height = height as u32;
        }
    }

    /// Move the coordinates in a positive direction while keeping the right and bottom edge.
    pub fn add_coordinate_abs<C>(&mut self, coord: C)
    where
        C: Into<Coordinate>,
    {
        let coord = coord.into();

        let right = self.0.right();
        let bottom = self.0.bottom();
        self.0.x = (self.0.x + coord.x).min(right);
        self.0.y = (self.0.y + coord.y).min(bottom);
        self.0.size = Size::new(right - self.0.x, bottom - self.0.y);
    }

    /// Get the amount of X pixels.
    pub fn width(&self) -> u32 {
        self.0.width()
    }

    /// Get the amount of Y pixels.
    pub fn height(&self) -> u32 {
        self.0.height()
    }

    /// X position.
    pub fn x(&self) -> i32 {
        self.0.x
    }

    /// Y position.
    pub fn y(&self) -> i32 {
        self.0.y
    }
    /// Right position, `x + width`.
    pub fn right(&self) -> i32 {
        self.0.right()
    }

    /// Bottom position, `y + height`.
    pub fn bottom(&self) -> i32 {
        self.0.bottom()
    }

    /// Get our data as the subrectangle.
    pub fn as_sub_rect(&self) -> Rect {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cull() {
        // Fully outside
        assert_eq!(ImageView::new((-10, 0, 10, 10), (0, 0, 100, 100)), None);
        assert_eq!(ImageView::new((0, -10, 10, 10), (0, 0, 100, 100)), None);
        assert_eq!(ImageView::new((100, 0, 10, 10), (0, 0, 100, 100)), None);
        assert_eq!(ImageView::new((0, 100, 10, 10), (0, 0, 100, 100)), None);
        assert_eq!(ImageView::new((0, 0, 10, 10), (10, 0, 100, 100)), None);
        assert_eq!(ImageView::new((0, 0, 10, 10), (0, 10, 100, 100)), None);
        assert_eq!(ImageView::new((110, 0, 10, 10), (10, 0, 100, 100)), None);
        assert_eq!(ImageView::new((0, 110, 10, 10), (0, 10, 100, 100)), None);
    }

    #[test]
    fn clip() {
        // Clip left
        assert_eq!(
            ImageView::new((-5, 0, 10, 10), (0, 0, 100, 100)),
            Some(ImageView::new_unchecked(0, 0, Size::new(5, 10)))
        );
        // Clip top
        assert_eq!(
            ImageView::new((0, -5, 10, 10), (0, 0, 100, 100)),
            Some(ImageView::new_unchecked(0, 0, Size::new(10, 5)))
        );
        // Clip right
        assert_eq!(
            ImageView::new((95, 0, 10, 10), (0, 0, 100, 100)),
            Some(ImageView::new_unchecked(95, 0, Size::new(5, 10)))
        );
        // Clip bottom
        assert_eq!(
            ImageView::new((0, 95, 10, 10), (0, 0, 100, 100)),
            Some(ImageView::new_unchecked(0, 95, Size::new(10, 5)))
        );
    }

    #[test]
    fn parent_ranges() {
        // Top left corner
        assert_eq!(
            ImageView::new_unchecked(0, 0, Size::new(10, 3))
                .parent_ranges_iter(Size::new(100, 100))
                .collect::<Vec<_>>(),
            vec![0..10, 100..110, 200..210]
        );

        // With some offset in the center
        assert_eq!(
            ImageView::new_unchecked(10, 10, Size::new(10, 3))
                .parent_ranges_iter(Size::new(100, 100))
                .collect::<Vec<_>>(),
            vec![1010..1020, 1110..1120, 1210..1220]
        );
    }

    #[test]
    fn add_coordinate_abs() {
        let mut rect = ImageView::new_unchecked(10, 10, (10, 10));
        rect.add_coordinate_abs((5, 2));

        assert_eq!(rect, ImageView::new_unchecked(15, 12, (5, 8)));
    }
}
