use aseprite::SpritesheetData;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{error::Result, BlitBuffer};

/// `BlitBuffer` with extra information for rendering as a scalable slice 9 graphic.
///
/// The slice information for scaling it should be exported in the output JSON from aseprite.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Slice9BlitBuffer {
    /// Full image.
    buffer: BlitBuffer,
    /// X position of both vertical slices.
    vertical_slices: (i32, i32),
    /// Y position of both horizontal slices.
    horizontal_slices: (i32, i32),
}

impl Slice9BlitBuffer {
    /// Construct a new buffer for animating a spritesheet.
    pub fn new(buffer: BlitBuffer, info: SpritesheetData) -> Self {
        todo!()
    }

    /// Draw the current frame using the animation info.
    pub fn blit(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        (offset_x, offset_y, width, height): (i32, i32, i32, i32),
    ) {
        todo!()
    }
}
