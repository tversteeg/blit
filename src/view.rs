use std::ops::Range;

use num_traits::ToPrimitive;

use crate::{Size, SubRect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImageView(pub(crate) SubRect);

impl ImageView {
    /// Create a new view based on the size of the original buffer.
    ///
    /// When it's `None` the view doesn't contain any actual pixels.
    pub fn new<T, P>(target: T, parent: P) -> Option<Self>
    where
        T: Into<SubRect>,
        P: Into<SubRect>,
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
    pub fn new_unchecked<X, Y>(x: X, y: Y, size: Size) -> Self
    where
        X: ToPrimitive,
        Y: ToPrimitive,
    {
        Self(SubRect::new(x, y, size))
    }

    /// Create a full view of a complete buffer without checking if it fits in the parent.
    pub fn full(size: Size) -> Self {
        Self(SubRect { x: 0, y: 0, size })
    }

    /// Iterator over horizontal ranges in the buffer the view is based on.
    ///
    /// Each range represents a slice of bytes that can be taken.
    /// Bounds checks should have already been done by the new function.
    pub fn parent_ranges_iter(&self, parent_size: Size) -> impl Iterator<Item = Range<usize>> {
        let (width, height) = (self.0.width() as usize, self.0.height() as usize);
        let (start_x, start_y) = (self.0.x as usize, self.0.y as usize);
        let end_y = start_y + height;

        let parent_width = parent_size.width as usize;

        (start_y..end_y).map(move |y| {
            let start_x = y * parent_width + start_x;
            let end_x = start_x + width;

            start_x..end_x
        })
    }

    /// Size in pixels.
    pub fn size(&self) -> Size {
        self.0.size
    }

    /// Create a sub-view based on this view.
    pub fn sub<S>(&self, target: S) -> Option<Self>
    where
        S: Into<SubRect>,
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

    /// Clip the view to fit in another one.
    pub fn clip<R>(&self, other: R) -> ImageView
    where
        R: Into<SubRect>,
    {
        let other = other.into();
        let mut new = self.clone();

        let (right, bottom) = (self.0.right(), self.0.bottom());

        // Clip the left edge
        if self.0.x < other.x {
            new.0.x = other.x;
            let width = (right - new.0.x).max(0);
            new.0.size.width = width as u32;
        }
        // Clip the top edge
        if self.0.y < other.y {
            new.0.y = other.y;
            let height = (bottom - new.0.y).max(0);
            new.0.size.height = height as u32;
        }
        // Clip the right edge
        if right > other.right() {
            let width = (other.right() - new.0.x).max(0);
            new.0.size.width = width as u32;
        }
        // Clip the bottom edge
        if bottom > other.bottom() {
            let height = (other.bottom() - new.0.y).max(0);
            new.0.size.height = height as u32;
        }

        new
    }

    /// Get the amount of X pixels.
    pub fn width(&self) -> u32 {
        self.0.width()
    }

    /// Get the amount of Y pixels.
    pub fn height(&self) -> u32 {
        self.0.height()
    }

    /// X Y coordinates.
    pub fn coord(&self) -> (i32, i32) {
        (self.0.x, self.0.y)
    }

    /// Get our data as the subrectangle.
    pub fn as_sub_rect(&self) -> SubRect {
        self.0
    }
}

impl Into<SubRect> for ImageView {
    fn into(self) -> SubRect {
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
    fn clip_fn() {
        assert_eq!(
            ImageView::new_unchecked(10, 10, Size::new(40, 40)).clip(ImageView::new_unchecked(
                20,
                20,
                Size::new(20, 20)
            )),
            ImageView::new_unchecked(20, 20, Size::new(20, 20))
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
}
