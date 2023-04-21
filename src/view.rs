use std::{num::NonZeroUsize, ops::Range};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImageView {
    /// Offset of the view on the original data.
    offset: (i32, i32),
    /// Size of the view on the data.
    size: (NonZeroUsize, NonZeroUsize),
}

impl ImageView {
    /// Create a new view based on the size of the original buffer.
    ///
    /// When it's `None` the view doesn't contain any actual pixels.
    pub fn new<P1, P2, S1, S2>(
        (target_x, target_y, target_width, target_height): (P1, P1, S1, S1),
        (parent_x, parent_y, parent_width, parent_height): (P2, P2, S2, S2),
    ) -> Option<Self>
    where
        P1: TryInto<i32>,
        P2: TryInto<i32>,
        S1: TryInto<i32>,
        S2: TryInto<i32>,
    {
        // Return `None` if any of the conversions failed
        let (target_x, target_y, target_width, target_height) = match (
            target_x.try_into(),
            target_y.try_into(),
            target_width.try_into(),
            target_height.try_into(),
        ) {
            (Ok(x), Ok(y), Ok(w), Ok(h)) => (x, y, w, h),
            _ => return None,
        };
        let (parent_x, parent_y, parent_width, parent_height) = match (
            parent_x.try_into(),
            parent_y.try_into(),
            parent_width.try_into(),
            parent_height.try_into(),
        ) {
            (Ok(x), Ok(y), Ok(w), Ok(h)) => (x, y, w, h),
            _ => return None,
        };

        let (parent_right, parent_bottom) = (parent_x + parent_width, parent_y + parent_height);

        // No pixels will be drawn when the view taken is outside of the bounds of the original image
        if target_x <= parent_x - target_width
            || target_y <= parent_y - target_height
            || target_x >= parent_right
            || target_y >= parent_bottom
        {
            return None;
        }

        // Clip the target to the left and top side
        let (offset_x, offset_y) = (target_x.max(parent_x), target_y.max(parent_x));

        // How much pixels are subtracted from the left and top side by clipping
        let (subtract_x, subtract_y) = (offset_x - target_x, offset_y - target_y);

        // Clip the width and height to right and bottom side if applicable
        let (actual_width, actual_height) = (
            (target_width - subtract_x).min(parent_right - target_x),
            (target_height - subtract_y).min(parent_bottom - target_y),
        );

        Self::new_unchecked((offset_x, offset_y, actual_width, actual_height))
    }

    /// Create a new view that's not clipped by it's parent.
    ///
    /// This is faster when you're 100% sure it fits.
    /// `width <= 0` and `height <= 0` will still be checked.
    pub fn new_unchecked<P, S>(
        (target_x, target_y, target_width, target_height): (P, P, S, S),
    ) -> Option<Self>
    where
        P: TryInto<i32>,
        S: TryInto<i32>,
    {
        let (target_x, target_y, target_width, target_height) = match (
            target_x.try_into(),
            target_y.try_into(),
            target_width.try_into(),
            target_height.try_into(),
        ) {
            (Ok(x), Ok(y), Ok(w), Ok(h)) => (x, y, w, h),
            _ => return None,
        };

        // No pixels will be drawn when the size is smaller than 0
        if target_width <= 0 || target_height <= 0 {
            return None;
        }

        // SAFETY: due to to above call this can never be zero or smaller
        let target_width = unsafe { NonZeroUsize::new_unchecked(target_width as usize) };
        let target_height = unsafe { NonZeroUsize::new_unchecked(target_height as usize) };

        let offset = (target_x, target_y);
        let size = (target_width, target_height);

        Some(Self { offset, size })
    }

    /// Create a sub-view based on this view.
    pub fn sub<P, S>(&self, target: (P, P, S, S)) -> Option<Self>
    where
        P: TryInto<i32>,
        S: TryInto<i32>,
    {
        Self::new(target, self.as_i32_slice())
    }

    /// Iterator over horizontal ranges in the buffer the view is based on.
    ///
    /// Each range represents a slice of bytes that can be taken.
    /// Bounds checks should have already been done by the new function.
    pub fn parent_ranges_iter(&self, parent_width: usize) -> impl Iterator<Item = Range<usize>> {
        let (width, height) = self.size;
        let (width, height) = (width.get(), height.get());

        let (start_x, start_y) = self.offset;
        let (start_x, start_y) = (start_x as usize, start_y as usize);
        let end_y = start_y + height;

        (start_y..end_y).into_iter().map(move |y| {
            let start_x = (y * parent_width + start_x) as usize;
            let end_x = start_x + width;

            start_x..end_x
        })
    }

    /// Get our data as a `(x, y, width, height)` slice.
    pub fn as_slice(&self) -> (i32, i32, NonZeroUsize, NonZeroUsize) {
        (self.offset.0, self.offset.1, self.size.0, self.size.1)
    }

    /// Get our data as a `(x, y, width, height)` slice with only `i32` values.
    pub fn as_i32_slice(&self) -> (i32, i32, i32, i32) {
        (
            self.offset.0,
            self.offset.1,
            self.size.0.get() as i32,
            self.size.1.get() as i32,
        )
    }

    /// Get the amount of X pixels.
    pub fn width(&self) -> usize {
        self.size.0.get()
    }

    /// Get the amount of Y pixels.
    pub fn height(&self) -> usize {
        self.size.1.get()
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
