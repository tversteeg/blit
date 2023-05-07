use crate::{geom::Coordinate, Blit, BlitPipeline};

/// Pipeline for rendering a source image with a UV offset on a target image.
///
/// Instantiated by [`BlitPipeline::uv`].
#[derive(Debug)]
pub struct UvOffset<'a, 'b, B: Blit> {
    /// Main part of the image to draw.
    middle: BlitPipeline<'a, 'b, B>,
    /// Left part to draw if negative U.
    left: Option<BlitPipeline<'a, 'b, B>>,
    /// Top part to draw if negative V.
    top: Option<BlitPipeline<'a, 'b, B>>,

    /// UV offset of the source buffer.
    uv: Coordinate,
}

impl<'a, 'b, B: Blit> UvOffset<'a, 'b, B> {
    /// Render the result.
    pub fn draw(&mut self) {
        if let Some(left) = &mut self.left {
            left.draw();
        }
        if let Some(top) = &mut self.top {
            top.draw();
        }
        self.middle.draw();
    }

    /// Construct from an existing pipeline.
    pub(crate) fn new(pipeline: BlitPipeline<'a, 'b, B>, uv: Coordinate) -> Self {
        // Make sure that any value properly wraps around
        let real_offset = uv % pipeline.source_view.size().as_coordinate();

        let middle = pipeline;
        let left = None;
        let top = None;

        Self {
            middle,
            uv,
            left,
            top,
        }
    }
}
