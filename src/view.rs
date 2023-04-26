use std::{
    num::{NonZeroU32, NonZeroUsize},
    ops::Range,
};

use num_traits::ToPrimitive;

use crate::{
    error::{Error, Result},
    Size, SubRect,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImageView(SubRect);

impl ImageView {
    /// Create a new view based on the size of the original buffer.
    ///
    /// When it's `None` the view doesn't contain any actual pixels.
    pub fn new<T, P>(target: T, parent: P) -> Option<Self>
    where
        T: TryInto<SubRect>,
        P: TryInto<SubRect>,
    {
        // Return `None` if any of the conversions failed
        let (target, parent) = match (target.try_into(), parent.try_into()) {
            (Ok(target), Ok(parent)) => (target, parent),
            _ => return None,
        };

        let (parent_right, parent_bottom) = (parent.right(), parent.bottom());

        // No pixels will be drawn when the view taken is outside of the bounds of the original image
        if target.x <= parent.x - target.width()
            || target.y <= parent.y - target.height()
            || target.x >= parent_right
            || target.y >= parent_bottom
        {
            return None;
        }

        // Clip the target to the left and top side
        let (offset_x, offset_y) = (target.x.max(parent.x), target.y.max(parent.y));

        // How much pixels are subtracted from the left and top side by clipping
        let (subtract_x, subtract_y) = (offset_x - target.x, offset_y - target.y);

        // Clip the width and height to right and bottom side if applicable
        let (actual_width, actual_height) = (
            (target.width() - subtract_x).min(parent_right - target.x),
            (target.height() - subtract_y).min(parent_bottom - target.y),
        );

        Self::try_from((target.x, target.y, actual_width, actual_height)).ok()
    }

    /// Create a full view of a complete buffer.
    pub fn full(size: Size) -> Self {
        Self(SubRect { x: 0, y: 0, size })
    }

    /// Create a sub-view based on this view.
    pub fn sub<S>(&self, target: S) -> Option<Self>
    where
        S: TryInto<SubRect>,
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
        let size = Size::new(new_width as u32, new_height as u32);

        // Taking a subrectangle failed when the size is zero
        if size.width == 0 || size.height == 0 {
            return None;
        }

        Some(Self(SubRect {
            x: clip_x as u32,
            y: clip_y as u32,
            size,
        }))
    }

    /// Iterator over horizontal ranges in the buffer the view is based on.
    ///
    /// Each range represents a slice of bytes that can be taken.
    /// Bounds checks should have already been done by the new function.
    pub fn parent_ranges_iter(&self, parent_width: usize) -> impl Iterator<Item = Range<usize>> {
        let (width, height) = self.0.size.as_tuple();
        let (start_x, start_y) = (self.0.x as usize, self.0.y as usize);
        let end_y = start_y + height as usize;

        (start_y..end_y).into_iter().map(move |y| {
            let start_x = (y * parent_width + start_x) as usize;
            let end_x = start_x + width as usize;

            start_x..end_x
        })
    }

    /// Get our data as a `(x, y, width, height)` slice.
    pub fn as_slice(&self) -> (u32, u32, u32, u32) {
        self.0.as_slice()
    }

    /// Get our data as the subrectangle.
    pub fn as_sub_rect(&self) -> SubRect {
        self.0
    }

    /// Get the amount of X pixels.
    pub fn width(&self) -> u32 {
        self.0.width()
    }

    /// Get the amount of Y pixels.
    pub fn height(&self) -> u32 {
        self.0.height()
    }

    /// Size in pixels.
    pub fn size(&self) -> Size {
        self.0.size
    }
}

impl From<SubRect> for ImageView {
    fn from(value: SubRect) -> Self {
        ImageView(value)
    }
}

impl<X, Y, W, H> From<(X, Y, W, H)> for ImageView
where
    X: ToPrimitive,
    Y: ToPrimitive,
    W: ToPrimitive,
    H: ToPrimitive,
{
    fn from(rect: (X, Y, W, H)) -> Self {
        Self(SubRect::from(rect))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipping() {
        // Fully outside
        assert_eq!(ImageView::new((-10, 0, 10, 10), (0, 0, 100, 100)), None,);
        assert_eq!(ImageView::new((0, -10, 10, 10), (0, 0, 100, 100)), None,);
        assert_eq!(ImageView::new((100, 0, 10, 10), (0, 0, 100, 100)), None,);
        assert_eq!(ImageView::new((0, 100, 10, 10), (0, 0, 100, 100)), None,);
        assert_eq!(ImageView::new((0, 0, 10, 10), (10, 0, 100, 100)), None,);
        assert_eq!(ImageView::new((0, 0, 10, 10), (0, 10, 100, 100)), None,);
        assert_eq!(ImageView::new((110, 0, 10, 10), (10, 0, 100, 100)), None,);
        assert_eq!(ImageView::new((0, 110, 10, 10), (0, 10, 100, 100)), None,);

        // Clip left
        assert_eq!(
            ImageView::new((-5, 0, 10, 10), (0, 0, 100, 100)),
            ImageView::new_unchecked((0, 0, 5, 10))
        );
        // Clip top
        assert_eq!(
            ImageView::new((0, -5, 10, 10), (0, 0, 100, 100)),
            ImageView::new_unchecked((0, 0, 10, 5))
        );
        // Clip right
        assert_eq!(
            ImageView::new((95, 0, 10, 10), (0, 0, 100, 100)),
            ImageView::new_unchecked((95, 0, 5, 10))
        );
        // Clip bottom
        assert_eq!(
            ImageView::new((0, 95, 10, 10), (0, 0, 100, 100)),
            ImageView::new_unchecked((0, 95, 10, 5))
        );
    }

    #[test]
    fn parent_ranges() {
        // Top left corner
        assert_eq!(
            ImageView::new_unchecked((0, 0, 10, 3))
                .unwrap()
                .parent_ranges_iter(100)
                .collect::<Vec<_>>(),
            vec![0..10, 100..110, 200..210]
        );

        // With some offset in the center
        assert_eq!(
            ImageView::new_unchecked((10, 10, 10, 3))
                .unwrap()
                .parent_ranges_iter(100)
                .collect::<Vec<_>>(),
            vec![1010..1020, 1110..1120, 1210..1220]
        );
    }
}
